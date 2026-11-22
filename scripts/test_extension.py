#!/usr/bin/env python3
import os
import re
import subprocess
import sys
import tempfile
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


def run_php(
    php: Path, profile: str, entry: Path, output: Path, *, load: bool = False,
    settings: tuple[str, ...] = (),
) -> subprocess.CompletedProcess[bytes]:
    command = [str(php), "-n", "-d", f"extension={EXTENSION}", "-d", f"zend_opcodes.output={output}"]
    for setting in (*PROFILES[profile], *settings):
        command.extend(["-d", setting])
    if load:
        command.extend(["-d", "zend_opcodes.load=1"])
    return subprocess.run(
        [*command, str(entry)], cwd=entry.parent, stdin=subprocess.DEVNULL,
        env={**os.environ, "ZEND_OPCODES_TEST_PROFILE": profile}, capture_output=True, timeout=30,
    )


def listing(output: Path, process: subprocess.CompletedProcess[bytes]) -> bytes:
    if process.returncode < 0 or not output.is_file() or output.stat().st_size == 0:
        raise ValueError(
            f"PHP status {process.returncode} while producing {output.name}\n"
            f"stdout: {process.stdout[:500]!r}\nstderr: {process.stderr[:500]!r}"
        )
    return output.read_bytes()


def compare(label: str, expected: bytes, actual: bytes) -> None:
    if expected != actual:
        for line, (left, right) in enumerate(
            zip_longest(expected.splitlines(keepends=True), actual.splitlines(keepends=True)), start=1
        ):
            if left != right:
                raise ValueError(f"{label} line {line}: expected {str(left)[:250]}, got {str(right)[:250]}")


def check_fixture(
    php: Path, profile: str, fixture: Path, directory: Path,
) -> None:
    source, expected = read_fixture(fixture)
    entry = directory / (fixture.with_suffix(".php").name if fixture.suffix == ".phpt" else fixture.name)
    entry.write_bytes(source)
    first = directory / "original.listing"
    second = directory / "reloaded.listing"
    error = fixture.with_name(f"{fixture.name}.error")
    stdout = fixture.with_name(f"{fixture.name}.out")
    if fixture.suffix == ".zop" and error.is_file():
        if stdout.is_file():
            raise ValueError("a listing cannot expect both successful output and loader rejection")
        reason = error.read_bytes().rstrip(b"\r\n")
        result = run_php(php, profile, entry, first, settings=("display_errors=stderr",))
        if (result.returncode <= 0 or result.stdout or first.exists()
                or not reason or reason not in result.stderr):
            raise ValueError(
                f"expected loader rejection {reason!r}, got status {result.returncode}, "
                f"stdout {result.stdout[:250]!r}, stderr {result.stderr[:250]!r}"
            )
        return

    reference = run_php(php, profile, entry, first)
    original = listing(first, reference)
    if fixture.suffix == ".zop" and stdout.is_file():
        compare("stdout", stdout.read_bytes(), reference.stdout)
        compare("stderr", b"", reference.stderr)
        if reference.returncode != 0:
            raise ValueError(f"expected exit status 0, got {reference.returncode}")
    if expected is not None:
        if reference.stdout and reference.stderr:
            raise ValueError("mixed stdout and stderr in PHPT EXPECT requires the upstream PHPT runner")
        whitespace = b" \t\n\r\x00\v"
        observed = reference.stdout or reference.stderr
        compare("PHPT EXPECT", expected.replace(b"\r\n", b"\n").strip(whitespace),
                observed.replace(b"\r\n", b"\n").strip(whitespace))

    entry.write_bytes(original)
    candidate = run_php(php, profile, entry, second, load=True)
    compare("opcode listing", original, listing(second, candidate))
    compare("stdout", reference.stdout, candidate.stdout)
    compare("stderr", reference.stderr, candidate.stderr)
    if reference.returncode != candidate.returncode:
        raise ValueError(f"exit status: expected {reference.returncode}, got {candidate.returncode}")


def main() -> int:
    php = Path(os.environ.get("ZEND_SPEC_PHP", ROOT.parent / "php-src/sapi/cli/php")).resolve()
    fixtures = sorted(
        path for path in TESTS.rglob("*") if path.suffix in {".php", ".phpt", ".zop"} and path.is_file()
    )
    if not php.is_file() or not EXTENSION.is_file() or not fixtures:
        sys.exit(f"Need a PHP executable at {php}, a built module at {EXTENSION}, and fixtures in {TESTS}.")
    failures = 0
    for fixture in fixtures:
        for profile in PROFILES:
            try:
                with tempfile.TemporaryDirectory(prefix="zend-opcodes-") as directory:
                    check_fixture(php, profile, fixture, Path(directory))
            except (OSError, ValueError, subprocess.TimeoutExpired) as error:
                print(f"FAIL {fixture.relative_to(ROOT)} [{profile}]: {error}", file=sys.stderr)
                failures += 1
    if failures:
        print(f"{failures} failed", file=sys.stderr)
        return 1
    print(f"{len(fixtures)} fixtures passed under both compiler profiles.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
