#!/usr/bin/env python3
import os
import re
import shutil
import subprocess
import sys
from dataclasses import dataclass
from itertools import zip_longest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
TESTS = ROOT / "tests"
EXTENSION = ROOT / "ext/modules/zend_opcodes.so"
PROFILES = {
    "ordinary": ("opcache.enable=0",),
    "default-optimized": (
        "opcache.enable=1",
        "opcache.enable_cli=1",
        "opcache.file_update_protection=0",
        "opcache.optimization_level=0x7FFEBFFF",
        "opcache.jit=disable",
    ),
}


def corpus() -> tuple[Path, list[Path]]:
    php = Path(os.environ.get("ZEND_SPEC_PHP", ROOT.parent / "php-src/sapi/cli/php")).resolve()
    fixtures = sorted(
        path for path in TESTS.rglob("*") if path.suffix in {".php", ".phpt", ".zop"} and path.is_file()
    )
    if not php.is_file() or not EXTENSION.is_file() or not fixtures:
        raise ValueError(f"Need a PHP executable at {php}, a built module at {EXTENSION}, and fixtures in {TESTS}.")
    return php, fixtures


def read_fixture(path: Path) -> tuple[bytes, bytes | None]:
    data = path.read_bytes()
    if path.suffix != ".phpt":
        return data, None
    parts = re.split(rb"^--([A-Z_]+)--[^\n]*(?:\n|$)", data, flags=re.MULTILINE)
    names = parts[1::2]
    if parts[0] or names[:1] != [b"TEST"] or sorted(names) != [b"EXPECT", b"FILE", b"TEST"]:
        raise ValueError(f"expected PHPT sections TEST, FILE and EXPECT, found {names!r}")
    sections = dict(zip(names, parts[2::2]))
    return sections[b"FILE"], sections[b"EXPECT"]


def expected_error(fixture: Path) -> bytes | None:
    error = fixture.with_name(f"{fixture.name}.error")
    if fixture.suffix != ".zop" or not error.is_file():
        return None
    if fixture.with_name(f"{fixture.name}.out").is_file():
        raise ValueError("a listing cannot expect both successful output and loader rejection")
    reason = error.read_bytes().rstrip(b"\r\n")
    if not reason:
        raise ValueError("expected loader diagnostic is empty")
    return reason


@dataclass
class Execution:
    stdout: bytes
    stderr: bytes
    status: int
    listing: bytes


def run_php(
    php: Path, profile: str, entry: Path, *,
    load: bool = False, rejection: bytes | None = None, compile_only: bool = False, stdin: bytes | None = None,
) -> Execution:
    output = entry.parent / "native.listing"
    output.unlink(missing_ok=True)
    command = [str(php), "-n", "-d", f"extension={EXTENSION}", "-d", f"zend_opcodes.output={output}"]
    for setting in PROFILES[profile]:
        command.extend(["-d", setting])
    if load:
        command.extend(["-d", "zend_opcodes.load=1"])
    if rejection is not None:
        command.extend(["-d", "display_errors=stderr"])
    if compile_only or rejection is not None:
        command.append("-l")
    result = subprocess.run(
        [*command, str(entry)], cwd=entry.parent, input=stdin, stdin=subprocess.DEVNULL if stdin is None else None,
        env={**os.environ, "ZEND_OPCODES_TEST_PROFILE": profile}, capture_output=True, timeout=30,
    )
    listing = output.read_bytes() if output.is_file() else b""
    if rejection is None:
        if result.returncode < 0 or (compile_only and result.returncode != 0) or not listing:
            raise ValueError(f"PHP status {result.returncode} producing listing: "
                             f"stdout {result.stdout[:500]!r}, stderr {result.stderr[:500]!r}")
    elif (result.returncode <= 0 or result.stdout != b"Errors parsing " + os.fsencode(entry) + b"\n"
          or output.exists() or rejection not in result.stderr):
        raise ValueError(f"expected loader rejection {rejection!r}, got status {result.returncode}, "
                         f"stdout {result.stdout[:250]!r}, stderr {result.stderr[:250]!r}")
    return Execution(result.stdout, result.stderr, result.returncode, listing)


def execute(
    php: Path, profile: str, fixture: Path, directory: Path, program: bytes, *,
    load: bool = False, rejection: bytes | None = None,
) -> Execution:
    run_dir = directory / "run"
    run_dir.mkdir()
    try:
        entry = run_dir / (fixture.with_suffix(".php").name if fixture.suffix == ".phpt" else fixture.name)
        entry.write_bytes(program)
        return run_php(php, profile, entry, load=load, rejection=rejection)
    finally:
        shutil.rmtree(run_dir)


def compare(label: str, expected: bytes, actual: bytes) -> None:
    if expected != actual:
        for line, (left, right) in enumerate(
            zip_longest(expected.splitlines(keepends=True), actual.splitlines(keepends=True)), start=1
        ):
            if left != right:
                raise ValueError(f"{label} line {line}: expected {str(left)[:250]}, got {str(right)[:250]}")


def check_expected(fixture: Path, result: Execution, expected: bytes | None) -> None:
    stdout = fixture.with_name(f"{fixture.name}.out")
    if fixture.suffix == ".zop" and stdout.is_file():
        compare("stdout", stdout.read_bytes(), result.stdout)
        compare("stderr", b"", result.stderr)
        if result.status != 0:
            raise ValueError(f"expected exit status 0, got {result.status}")
    if expected is not None:
        if result.stdout and result.stderr:
            raise ValueError("mixed stdout and stderr in PHPT EXPECT requires the upstream PHPT runner")
        whitespace = b" \t\n\r\x00\v"
        observed = result.stdout or result.stderr
        compare("PHPT EXPECT", expected.replace(b"\r\n", b"\n").strip(whitespace),
                observed.replace(b"\r\n", b"\n").strip(whitespace))


def compare_execution(reference: Execution, candidate: Execution) -> None:
    compare("opcode listing", reference.listing, candidate.listing)
    compare("stdout", reference.stdout, candidate.stdout)
    compare("stderr", reference.stderr, candidate.stderr)
    if reference.status != candidate.status:
        raise ValueError(f"exit status: expected {reference.status}, got {candidate.status}")


def prepare(php: Path, profile: str, fixture: Path, directory: Path) -> None:
    directory.mkdir()
    source, expected = read_fixture(fixture)
    rejection = expected_error(fixture)
    if rejection is not None:
        (directory / "expected-error").write_bytes(rejection)
    if fixture.suffix != ".zop":
        reference = execute(php, profile, fixture, directory, source)
        check_expected(fixture, reference, expected)
        source = reference.listing
    (directory / "original.listing").write_bytes(source)


def verify(php: Path, profile: str, fixture: Path, directory: Path) -> None:
    rejection = expected_error(fixture)
    original = (directory / "original.listing").read_bytes()
    reference = execute(php, profile, fixture, directory, original, load=True, rejection=rejection)
    printed_path = directory / "printed.listing"
    if (directory / "parse-error").is_file():
        if rejection is None or printed_path.exists():
            raise ValueError("unexpected Rust parse rejection")
        return
    check_expected(fixture, reference, None)
    printed = printed_path.read_bytes()
    candidate = execute(php, profile, fixture, directory, printed, load=True, rejection=rejection)
    if rejection is None:
        compare_execution(reference, candidate)


def main() -> int:
    if len(sys.argv) != 3 or sys.argv[1] not in {"prepare", "verify"}:
        sys.exit(f"usage: {Path(__file__).name} prepare|verify DIRECTORY")
    operation = prepare if sys.argv[1] == "prepare" else verify
    root = Path(sys.argv[2]).resolve()
    php, fixtures = corpus()
    index = 0
    for fixture in fixtures:
        for profile in PROFILES:
            directory = root / str(index)
            label = f"{fixture.relative_to(ROOT)} [{profile}]"
            try:
                if operation is verify and (directory / "label").read_bytes() != os.fsencode(label):
                    raise ValueError("fixture or profile changed after preparation")
                operation(php, profile, fixture, directory)
                if operation is prepare:
                    (directory / "label").write_bytes(os.fsencode(label))
            except (OSError, ValueError, subprocess.TimeoutExpired) as error:
                sys.exit(f"FAIL {label}: {error}")
            index += 1
    if operation is verify and len(list(root.iterdir())) != index:
        sys.exit("fixture set changed after preparation")
    return 0


if __name__ == "__main__":
    try:
        sys.exit(main())
    except (OSError, ValueError) as error:
        sys.exit(str(error))
