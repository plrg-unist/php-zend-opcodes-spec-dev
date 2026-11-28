#!/usr/bin/env python3
import argparse
import errno
import json
import math
import os
import re
import struct
import subprocess
import sys
import tempfile
from concurrent.futures import ThreadPoolExecutor, as_completed
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any

sys.dont_write_bytecode = True
from zend_fixtures import PROFILES, ROOT, TESTS, compare, corpus, expected_error, read_fixture, run_php


Observation = bytes | int
Divergence = dict[str, tuple[Observation, Observation]]


@dataclass
class BaseInputs:
    opline_size_bytes: int
    constants: dict[str, Any]
    names: dict[str, str]
    aliases: dict[str, str]


@dataclass
class Case:
    stdin: Path | None
    kind: str
    expected: dict[str, Observation]
    divergence: Divergence
    compare_zend: bool
    spec_flags: tuple[str, ...] | None
    inputs: dict[str, Any]


@dataclass
class Coverage:
    steps: list[str] = field(default_factory=list)
    hits: set[int] = field(default_factory=set)

    def merge(self, steps: list[str], hits: set[int]) -> None:
        if not steps:
            return
        if self.steps and self.steps != steps:
            raise ValueError("definition or step order changed across runs")
        self.steps = steps
        self.hits.update(hits)


def octets(value: object) -> bytes:
    if not isinstance(value, list) or any(type(byte) is not int for byte in value):
        raise ValueError(f"expected an array of octets, got {value!r}")
    return bytes(value)


def observation(name: str, value: object) -> Observation:
    if name == "exit_status":
        if type(value) is not int or not 0 <= value <= 255:
            raise ValueError(f"invalid exit_status {value!r}")
        return value
    return octets(value)


def read_divergence(value: object) -> Divergence:
    if not isinstance(value, dict) or value.keys() - {"rationale", "stdout", "stderr", "exit_status"}:
        raise ValueError(f"invalid divergence fields in {value!r}")
    rationale = value.get("rationale")
    if not isinstance(rationale, str) or not Path(rationale).is_relative_to("design-docs"):
        raise ValueError("divergence requires a rationale path under design-docs/")
    document = (ROOT / rationale).resolve()
    if not document.is_relative_to(ROOT / "design-docs") or document.suffix != ".md" or not document.is_file():
        raise ValueError(f"invalid divergence rationale {rationale!r}")
    fields: Divergence = {}
    for name, pair in value.items():
        if name == "rationale":
            continue
        if not isinstance(pair, dict) or pair.keys() != {"zend", "spec"}:
            raise ValueError(f"divergence {name} requires exact zend and spec values")
        zend = observation(name, pair["zend"])
        spec = observation(name, pair["spec"])
        if zend == spec:
            raise ValueError(f"divergence {name} must differ between Zend and the spec")
        fields[name] = (zend, spec)
    if not fields:
        raise ValueError("divergence requires at least one differing field")
    return fields


def unique_fields(pairs: list[tuple[str, Any]]) -> dict[str, Any]:
    fields = {}
    for name, value in pairs:
        if name in fields:
            raise ValueError(f"duplicate JSON field {name!r}")
        fields[name] = value
    return fields


def constant_key(name: str) -> str:
    # Zend folds ASCII letters in the namespace prefix preceding the first NUL.
    raw = name.encode()
    if raw.lower() in (b"true", b"false", b"null"):
        raise ValueError(f"constant {name!r} is a core PHP literal")
    end = raw.partition(b"\0")[0].rfind(b"\\") + 1
    return (raw[:end].lower() + raw[end:]).decode()


def constant_value_key(value: Any) -> tuple[object, object]:
    if value is None or type(value) is bool or (type(value) is int and -(1 << 63) <= value < 1 << 63):
        return type(value), value
    if type(value) is float and math.isfinite(value):
        return float, struct.pack(">d", value).hex()
    if isinstance(value, str):
        return bytes, value.encode()
    if isinstance(value, dict) and len(value) == 1:
        kind, item = next(iter(value.items()))
        if kind == "bytes":
            return bytes, octets(item)
        if kind == "float_bits" and isinstance(item, str) and re.fullmatch("[0-9a-fA-F]{16}", item):
            return float, item.lower()
        if kind == "stream" and type(item) is int and 0 < item < 1 << 63:
            return kind, item
    raise ValueError(f"invalid runtime constant value {value!r}")


def opline_size(value: object) -> int:
    if type(value) is not int or not 1 <= value <= 2**31 - 1:
        raise ValueError("opline_size_bytes must be a JSON integer from 1 to 2147483647")
    return value


def read_inputs(path: Path) -> BaseInputs:
    data = json.loads(path.read_text(encoding="utf-8"), object_pairs_hook=unique_fields)
    if not isinstance(data, dict) or data.keys() != {"constants", "opline_size_bytes"}:
        raise ValueError(f"{path}: runtime inputs require constants and opline_size_bytes")
    size = opline_size(data["opline_size_bytes"])
    entries = data["constants"]
    if not isinstance(entries, dict):
        raise ValueError(f"expected a runtime constants object in {path}")
    names = {}
    for name in entries:
        key = constant_key(name)
        if key in names:
            raise ValueError(f"{path}: duplicate registered constant {name!r}")
        names[key] = name
    aliases = {}
    for name, entry in entries.items():
        if isinstance(entry, dict) and "alias" in entry:
            if entry.keys() != {"alias"} or not isinstance(entry["alias"], str):
                raise ValueError(f"{path}: invalid alias definition for {name!r}")
            target = names.get(constant_key(entry["alias"]))
            if target is None:
                raise ValueError(f"{path}: missing alias target for {name!r}")
            aliases[name] = target
            continue
        if isinstance(entry, dict) and "value" in entry:
            metadata = entry.get("deprecated")
            if (entry.keys() != {"value", "deprecated"} or not isinstance(metadata, dict)
                    or metadata.keys() - {"since", "message"}
                    or not all(isinstance(text, str) for text in metadata.values())):
                raise ValueError(f"{path}: invalid deprecation for {name!r}")
            for text in metadata.values():
                text.encode("utf-8")
            entry = entry["value"]
        try:
            constant_value_key(entry)
        except ValueError as error:
            raise ValueError(f"{path}: constant {name!r}: {error}") from error
    if any(target in aliases for target in aliases.values()):
        raise ValueError(f"{path}: a constant alias must refer directly to a value")
    return BaseInputs(size, entries, names, aliases)


def case_inputs(
    base: BaseInputs, overrides: object, path: Path | None,
) -> tuple[tuple[str, ...] | None, dict[str, Any]]:
    if not isinstance(overrides, dict) or overrides.keys() - {"constants", "opline_size_bytes"}:
        raise ValueError("inputs must be an object with known runtime input fields")
    constants = overrides.get("constants", {})
    if not isinstance(constants, dict):
        raise ValueError("constants must be an object")
    flags = ["--inputs", str(path)] if path else []
    size = base.opline_size_bytes
    if "opline_size_bytes" in overrides:
        size = opline_size(overrides["opline_size_bytes"])
        flags.extend(("--input", f"opline_size_bytes={size}"))
    names = base.names.copy()
    values = base.constants.copy()
    changes = {}
    # An OS argument cannot contain NUL. The first '=' terminates the CLI name.
    file_required = any("\0" in name or "=" in name for name in constants)
    for name, value in constants.items():
        if not file_required:
            flags.extend(("--input", "constants." + name + "=" + json.dumps(value, allow_nan=False)))
        spelling = names.setdefault(constant_key(name), name)
        target = base.aliases.get(spelling, spelling)
        key = constant_value_key(value)
        if target in changes and changes[target] != key:
            raise ValueError(f"conflicting values for constant {name!r}")
        changes[target] = key
        entry = values.get(target)
        if isinstance(entry, dict) and "deprecated" in entry:
            values[target] = {**entry, "value": value}
        else:
            values[target] = value
    for alias, target in base.aliases.items():
        values[alias] = values[target]
    return None if file_required else tuple(flags), {"opline_size_bytes": size, "constants": values}


def read_cases(fixtures: list[Path]) -> dict[Path, list[Case]]:
    groups: dict[Path, list[Case]] = {}
    bases: dict[Path | None, BaseInputs] = {None: read_inputs(ROOT / "runtime-inputs.json")}
    rows = json.loads((TESTS / "semantics.json").read_text(encoding="utf-8"), object_pairs_hook=unique_fields)
    if not isinstance(rows, list):
        raise ValueError("expected a semantics manifest array")
    for row in rows:
        if (not isinstance(row, dict) or not isinstance(row.get("fixture"), str)
                or not isinstance(row.get("stdin", ""), str)):
            raise ValueError(f"invalid semantics case {row!r}")
        fixture = TESTS / row["fixture"]
        stdin = TESTS / row["stdin"] if "stdin" in row else None
        kind = row.get("expect", "php")
        if fixture not in fixtures or kind not in ("php", "unsupported", "malformed"):
            raise ValueError(f"invalid semantics case {row!r}")
        if stdin is not None and (not stdin.resolve().is_relative_to(TESTS) or not stdin.is_file()):
            raise ValueError(f"unknown stdin file {stdin}")
        if set(row) - {"fixture", "stdin", "expect", "stdout", "stderr", "exit_status", "divergence",
                       "inputs", "inputs_file", "compare_zend"}:
            raise ValueError(f"unknown semantics case fields in {row!r}")
        if kind != "malformed" and expected_error(fixture) is not None:
            raise ValueError(f"native rejection fixture requires a malformed expectation: {fixture}")
        expected = {
            name: value.encode() if name != "exit_status" and isinstance(value, str) else observation(name, value)
            for name, value in row.items() if name in ("stdout", "stderr", "exit_status")
        }
        if (kind == "malformed" and expected) or (kind != "php" and "exit_status" in expected):
            raise ValueError(f"invalid observations for a {kind} case")
        compare_zend = row.get("compare_zend", kind == "php")
        if type(compare_zend) is not bool or (compare_zend and kind != "php"):
            raise ValueError("compare_zend must be boolean and can be true only for php outcomes")
        if "divergence" in row and not compare_zend:
            raise ValueError("divergence requires a native Zend comparison")
        if kind == "php" and not compare_zend and expected.keys() != {"stdout", "stderr", "exit_status"}:
            raise ValueError("a spec-only php case requires stdout, stderr and exit_status")
        divergence = read_divergence(row["divergence"]) if "divergence" in row else {}
        if expected.keys() & divergence.keys():
            raise ValueError("an observation cannot have both a shared and a divergent expectation")
        path = None
        if "inputs_file" in row:
            if not isinstance(row["inputs_file"], str) or Path(row["inputs_file"]).is_absolute():
                raise ValueError("inputs_file must be a path relative to tests/")
            path = (TESTS / row["inputs_file"]).resolve()
            if not path.is_relative_to(TESTS) or not path.is_file():
                raise ValueError(f"unknown inputs file {path}")
            if path not in bases:
                bases[path] = read_inputs(path)
        flags, inputs = case_inputs(bases[path], row.get("inputs", {}), path)
        groups.setdefault(fixture, []).append(Case(stdin, kind, expected, divergence, compare_zend, flags, inputs))
    return groups


def select_cases(cases: dict[Path, list[Case]], paths: list[str]) -> dict[Path, list[Case]]:
    if not paths:
        return cases
    registered = {fixture.resolve() for fixture in cases}
    selected: set[Path] = set()
    for path in paths:
        if not os.path.isfile(path):
            raise ValueError(f"expected an existing fixture file: {path}")
        resolved = Path(path).resolve()
        if resolved not in registered:
            raise ValueError(f"no registered semantics case in tests/semantics.json for {path}")
        selected.add(resolved)
    return {fixture: variants for fixture, variants in cases.items() if fixture.resolve() in selected}


def build_binary() -> Path:
    build = subprocess.run(
        ["cargo", "build", "--bin", "zend-opcodes", "--message-format=json"], cwd=ROOT, stdout=subprocess.PIPE,
    )
    messages = [json.loads(line) for line in build.stdout.splitlines()]
    if build.returncode:
        for message in messages:
            if message["reason"] == "compiler-message":
                print(message["message"]["rendered"], end="", file=sys.stderr)
        raise ValueError("cargo build failed")
    for message in messages:
        if (message["reason"] == "compiler-artifact" and message["target"]["name"] == "zend-opcodes"
                and message["executable"]):
            return Path(message["executable"])
    raise ValueError("cargo did not report the zend-opcodes executable")


def json_outcome(
    binary: Path, listing: Path, stdin: bytes, coverage: Path | None = None, flags: tuple[str, ...] = (),
) -> dict[str, Any]:
    env = os.environ.copy()
    if coverage is not None:
        coverage.unlink(missing_ok=True)
        env["ZEND_SPEC_COVERAGE"] = str(coverage)
    result = subprocess.run(
        [binary, "--json", *flags, listing], cwd=listing.parent, input=stdin, capture_output=True, env=env, timeout=30,
    )
    if result.returncode or result.stderr:
        raise ValueError(f"JSON mode status {result.returncode}, stderr {result.stderr[:500]!r}")
    outcome = json.loads(result.stdout)
    if not isinstance(outcome, dict):
        raise ValueError("JSON mode must report an object")
    return outcome


def coverage_hits(report: Path) -> tuple[list[str], set[int]]:
    lines = report.read_text().splitlines()
    header = re.match(r";; Instruction coverage: (\d+)/(\d+) ", lines[0] if lines else "")
    steps: list[str] = []
    hits: set[int] = set()
    owner = ""
    for line in lines:
        if line.startswith(("def ", "relation ", "tbl def ")):
            owner = line
        if line.startswith(("+ ", "- ")):
            if not owner:
                raise ValueError("coverage step without an owning definition")
            if line.startswith("+ "):
                hits.add(len(steps))
            step = re.sub(r"  \[hits: \d+\]$", "", line[2:])
            steps.append(f"{owner}  {step}")
    if not header or not steps or (len(hits), len(steps)) != tuple(map(int, header.groups())):
        raise ValueError("invalid or empty instruction coverage report")
    return steps, hits


def check_observation(label: str, expected: Observation, actual: Observation) -> None:
    if isinstance(expected, bytes) and isinstance(actual, bytes):
        compare(label, expected, actual)
    elif actual != expected:
        raise ValueError(f"{label}: expected {expected!r}, got {actual!r}")


def check_case(binary: Path, php: Path, profile: str, listing: Path, case: Case, report: Path) -> None:
    stdin = case.stdin.read_bytes() if case.stdin else b""
    flags = case.spec_flags
    if flags is not None:
        try:
            outcome = json_outcome(binary, listing, stdin, report, flags)
        except OSError as error:
            if not flags or error.errno != errno.E2BIG:
                raise
            flags = None
    if flags is None:
        path = listing.parent / "spec-inputs.json"
        path.write_text(json.dumps(case.inputs, allow_nan=False), encoding="utf-8")
        flags = ("--inputs", str(path))
        outcome = json_outcome(binary, listing, stdin, report, flags)
    if outcome.get("kind") != case.kind:
        raise ValueError(f"expected {case.kind}, got {outcome!r}")
    normal = subprocess.run(
        [binary, *flags, listing], cwd=listing.parent, input=stdin, capture_output=True, timeout=30,
    )
    if case.kind == "php":
        spec_expected = case.expected.copy()
        if case.compare_zend:
            reference = run_php(php, profile, listing, load=True, stdin=stdin, constants=case.inputs["constants"])
            for name, zend in (
                ("stdout", reference.stdout), ("stderr", reference.stderr), ("exit_status", reference.status),
            ):
                shared = case.expected.get(name, zend)
                expected_zend, spec_expected[name] = case.divergence.get(name, (shared, shared))
                check_observation(f"Zend {name}", expected_zend, zend)
        for name, actual in (
            ("stdout", normal.stdout), ("stderr", normal.stderr), ("exit_status", normal.returncode),
        ):
            check_observation(f"JSON {name}", spec_expected[name], observation(name, outcome[name]))
            check_observation(f"normal {name}", spec_expected[name], actual)
    else:
        stdout = case.expected.get("stdout", b"")
        stderr = case.expected.get("stderr", b"")
        if case.kind == "unsupported":
            compare("JSON stdout prefix", stdout, octets(outcome["stdout"]))
            compare("JSON stderr prefix", stderr, octets(outcome["stderr"]))
        elif report.exists() or not outcome.get("message") or outcome.get("stdout", []) or outcome.get("stderr", []):
            raise ValueError(f"expected a pre-execution malformed result, got {outcome!r}")
        compare("stdout before panic", stdout, normal.stdout)
        if normal.returncode != 101 or not normal.stderr.startswith(stderr) or normal.stderr == stderr:
            raise ValueError(f"expected panic after the emitted prefix, got {normal!r}")


def check_profile(binary: Path, php: Path, fixture: Path, profile: str, variants: list[Case]) -> Coverage:
    name = fixture.relative_to(TESTS)
    label = f"{name} [{profile}]"
    coverage = Coverage()
    try:
        with tempfile.TemporaryDirectory(prefix="zend-semantics-") as scratch:
            directory = Path(scratch)
            listing = directory / "program.zop"
            report = directory / "coverage.txt"
            data, _ = read_fixture(fixture)
            if fixture.suffix != ".zop":
                source = directory / fixture.with_suffix(".php").name
                source.write_bytes(data)
                data = run_php(php, profile, source, compile_only=True).listing
            listing.write_bytes(data)
            for number, case in enumerate(variants, 1):
                label = f"{name} [{profile}, case {number}, {case.stdin.name if case.stdin else 'empty stdin'}]"
                check_case(binary, php, profile, listing, case, report)
                if case.kind != "malformed":
                    coverage.merge(*coverage_hits(report))
    except (OSError, ValueError, KeyError, subprocess.TimeoutExpired) as error:
        raise ValueError(f"{label}: {error}") from error
    return coverage


def main() -> None:
    parser = argparse.ArgumentParser(description="Compare Zend and spec executions and measure spec coverage.")
    parser.add_argument("-j", dest="jobs", type=int, default=os.cpu_count() or 1, metavar="JOBS",
                        help="maximum parallel jobs (default: CPU count, or 1 if unknown)")
    parser.add_argument("paths", nargs="*", metavar="PATH",
                        help="registered fixture paths (default: full suite with 100%% coverage required)")
    args = parser.parse_args()
    if args.jobs < 1:
        parser.error("-j requires a positive integer")

    os.environ.pop("ZEND_SPEC_COVERAGE", None)
    php, fixtures = corpus()
    cases = select_cases(read_cases(fixtures), args.paths)
    binary = build_binary()
    if not args.paths:
        with tempfile.TemporaryDirectory(prefix="zend-semantics-") as scratch:
            missing = json_outcome(binary, Path(scratch) / "program.zop", b"")
            if missing.get("kind") != "tool_error" or not missing.get("message"):
                raise ValueError(f"expected a tool_error for a missing file, got {missing!r}")

    coverage = Coverage()
    with ThreadPoolExecutor(max_workers=args.jobs) as pool:
        pending = {
            pool.submit(check_profile, binary, php, fixture, profile, variants): (fixture, profile)
            for fixture, variants in cases.items() for profile in PROFILES
        }
        try:
            for future in as_completed(pending):
                fixture, profile = pending.pop(future)
                result = future.result()
                try:
                    coverage.merge(result.steps, result.hits)
                except ValueError as error:
                    raise ValueError(f"{fixture.relative_to(TESTS)} [{profile}]: {error}") from error
        finally:
            for future in pending:
                future.cancel()

    summary = f"{len(coverage.hits)}/{len(coverage.steps)} SL instructions"
    if not args.paths and (not coverage.steps or len(coverage.hits) != len(coverage.steps)):
        for index, step in enumerate(coverage.steps):
            if index not in coverage.hits:
                print(f"uncovered #{index}: {step}", file=sys.stderr)
        raise ValueError(f"incomplete coverage: {summary}")
    if not coverage.steps:
        summary = "0 SL instructions executed (all selected cases were malformed)"
    count = sum(len(variants) for variants in cases.values()) * len(PROFILES)
    print(f"{count} semantics cases passed. Coverage: {summary}.")


if __name__ == "__main__":
    try:
        main()
    except (OSError, ValueError, KeyError, subprocess.TimeoutExpired) as error:
        sys.exit(f"FAIL {error}")
