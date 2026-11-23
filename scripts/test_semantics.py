#!/usr/bin/env python3
import json
import os
import re
import subprocess
import sys
import tempfile
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any

sys.dont_write_bytecode = True
from zend_fixtures import PROFILES, ROOT, TESTS, compare, corpus, expected_error, read_fixture, run_php


Observation = bytes | int
Divergence = dict[str, tuple[Observation, Observation]]


@dataclass
class Case:
    stdin: Path | None
    kind: str
    stdout: bytes
    stderr: bytes
    divergence: Divergence = field(default_factory=dict)


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


def read_cases(fixtures: list[Path]) -> dict[Path, list[Case]]:
    groups: dict[Path, list[Case]] = {}
    rows = json.loads((TESTS / "semantics.json").read_text())
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
        if set(row) - {"fixture", "stdin", "expect", "stdout", "stderr", "divergence"}:
            raise ValueError(f"unknown semantics case fields in {row!r}")
        if kind != "malformed" and expected_error(fixture) is not None:
            raise ValueError(f"native rejection fixture requires a malformed expectation: {fixture}")
        if kind != "unsupported" and ({"stdout", "stderr"} & row.keys()):
            raise ValueError("stream expectations in the manifest belong to unsupported cases")
        if kind != "php" and "divergence" in row:
            raise ValueError("divergence requires a php expectation")
        groups.setdefault(fixture, []).append(Case(
            stdin, kind, octets(row.get("stdout", [])), octets(row.get("stderr", [])),
            read_divergence(row["divergence"]) if "divergence" in row else {},
        ))
    return groups


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


def json_outcome(binary: Path, listing: Path, stdin: bytes, coverage: Path | None = None) -> dict[str, Any]:
    env = os.environ.copy()
    if coverage is not None:
        coverage.unlink(missing_ok=True)
        env["ZEND_SPEC_COVERAGE"] = str(coverage)
    result = subprocess.run(
        [binary, "--json", listing], cwd=listing.parent, input=stdin, capture_output=True, env=env, timeout=30,
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
    outcome = json_outcome(binary, listing, stdin, report)
    if outcome.get("kind") != case.kind:
        raise ValueError(f"expected {case.kind}, got {outcome!r}")
    normal = subprocess.run([binary, listing], cwd=listing.parent, input=stdin, capture_output=True, timeout=30)
    if case.kind == "php":
        reference = run_php(php, profile, listing, load=True, stdin=stdin)
        for name, zend, spec_normal in (
            ("stdout", reference.stdout, normal.stdout),
            ("stderr", reference.stderr, normal.stderr),
            ("exit_status", reference.status, normal.returncode),
        ):
            expected_zend, expected_spec = case.divergence.get(name, (zend, zend))
            check_observation(f"Zend {name}", expected_zend, zend)
            check_observation(f"JSON {name}", expected_spec, observation(name, outcome[name]))
            check_observation(f"normal {name}", expected_spec, spec_normal)
    else:
        if case.kind == "unsupported":
            compare("JSON stdout prefix", case.stdout, octets(outcome["stdout"]))
            compare("JSON stderr prefix", case.stderr, octets(outcome["stderr"]))
        elif report.exists() or not outcome.get("message") or outcome.get("stdout", []) or outcome.get("stderr", []):
            raise ValueError(f"expected a pre-execution malformed result, got {outcome!r}")
        compare("stdout before panic", case.stdout, normal.stdout)
        if normal.returncode != 101 or not normal.stderr.startswith(case.stderr) or normal.stderr == case.stderr:
            raise ValueError(f"expected panic after the emitted prefix, got {normal!r}")


def main() -> None:
    os.environ.pop("ZEND_SPEC_COVERAGE", None)
    php, fixtures = corpus()
    cases = read_cases(fixtures)
    binary = build_binary()
    all_steps: list[str] = []
    all_hits: set[int] = set()
    count = 0
    with tempfile.TemporaryDirectory(prefix="zend-semantics-") as scratch:
        directory = Path(scratch)
        listing = directory / "program.zop"
        report = directory / "coverage.txt"
        missing = json_outcome(binary, listing, b"")
        if missing.get("kind") != "tool_error" or not missing.get("message"):
            raise ValueError(f"expected a tool_error for a missing file, got {missing!r}")
        for fixture, variants in cases.items():
            data, _ = read_fixture(fixture)
            for profile in PROFILES:
                label = f"{fixture.relative_to(TESTS)} [{profile}]"
                try:
                    if fixture.suffix == ".zop":
                        listing.write_bytes(data)
                    else:
                        source = directory / fixture.with_suffix(".php").name
                        source.write_bytes(data)
                        listing.write_bytes(run_php(php, profile, source, compile_only=True).listing)
                    for case in variants:
                        label = f"{fixture.relative_to(TESTS)} [{profile}, {case.stdin.name if case.stdin else 'empty stdin'}]"
                        check_case(binary, php, profile, listing, case, report)
                        if case.kind != "malformed":
                            steps, hits = coverage_hits(report)
                            if all_steps and all_steps != steps:
                                raise ValueError("definition or step order changed across runs")
                            all_steps = steps
                            all_hits.update(hits)
                        count += 1
                except (OSError, ValueError, KeyError, subprocess.TimeoutExpired) as error:
                    raise ValueError(f"{label}: {error}") from error
    for index, step in enumerate(all_steps):
        if index not in all_hits:
            print(f"uncovered #{index}: {step}", file=sys.stderr)
    coverage = f"{len(all_hits)}/{len(all_steps)} SL instructions"
    if not all_steps or len(all_hits) != len(all_steps):
        raise ValueError(f"incomplete coverage: {coverage}")
    print(f"{count} semantics cases passed. Coverage: {coverage}.")


if __name__ == "__main__":
    try:
        main()
    except (OSError, ValueError, KeyError, subprocess.TimeoutExpired) as error:
        sys.exit(f"FAIL {error}")
