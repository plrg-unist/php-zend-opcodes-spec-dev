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
    if path.suffix == ".php":
        return data, None
    parts = re.split(rb"^--([A-Z_]+)--[^\n]*(?:\n|$)", data, flags=re.MULTILINE)
    names = parts[1::2]
    if parts[0] or names[:1] != [b"TEST"] or sorted(names) != [b"EXPECT", b"FILE", b"TEST"]:
        raise ValueError(f"expected PHPT sections TEST, FILE and EXPECT, found {names!r}")
    sections = dict(zip(names, parts[2::2]))
    return sections[b"FILE"], sections[b"EXPECT"]


def run_php(
    php: Path, settings: tuple[str, ...], entry: Path, output: Path, *, load: bool = False
) -> subprocess.CompletedProcess[bytes]:
    command = [str(php), "-n", "-d", f"extension={EXTENSION}", "-d", f"zend_opcodes.output={output}"]
    for setting in settings:
        command.extend(["-d", setting])
    if load:
        command.extend(["-d", "zend_opcodes.load=1"])
    return subprocess.run(
        [*command, str(entry)], cwd=entry.parent, stdin=subprocess.DEVNULL,
        capture_output=True, timeout=30,
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
    php: Path, settings: tuple[str, ...], entry: Path, source: bytes, expected: bytes | None,
) -> bytes:
    entry.write_bytes(source)
    first = entry.parent / "original.zop"
    second = entry.parent / "reloaded.zop"
    first.unlink(missing_ok=True)
    second.unlink(missing_ok=True)
    reference = run_php(php, settings, entry, first)
    original = listing(first, reference)
    if expected is not None:
        if reference.returncode != 0 or reference.stderr:
            raise ValueError(f"PHPT reference status {reference.returncode}, stderr {reference.stderr[:500]!r}")
        whitespace = b" \t\n\r\x00\v"
        compare("PHPT EXPECT", expected.replace(b"\r\n", b"\n").strip(whitespace),
                reference.stdout.replace(b"\r\n", b"\n").strip(whitespace))

    entry.write_bytes(original)
    candidate = run_php(php, settings, entry, second, load=True)
    compare("opcode listing", original, listing(second, candidate))
    compare("stdout", reference.stdout, candidate.stdout)
    compare("stderr", reference.stderr, candidate.stderr)
    if reference.returncode != candidate.returncode:
        raise ValueError(f"exit status: expected {reference.returncode}, got {candidate.returncode}")
    return original


def main() -> int:
    php = Path(os.environ.get("ZEND_SPEC_PHP", ROOT.parent / "php-src/sapi/cli/php")).resolve()
    fixtures = sorted([*TESTS.rglob("*.phpt"), *(TESTS / "project").rglob("*.php")])
    if not php.is_file() or not EXTENSION.is_file() or not fixtures:
        sys.exit(f"Need a PHP executable at {php}, a built module at {EXTENSION}, and fixtures in {TESTS}.")
    failures = 0
    for fixture in fixtures:
        label = str(fixture.relative_to(ROOT))
        try:
            source, expected = read_fixture(fixture)
        except (OSError, ValueError) as error:
            print(f"FAIL {label}: {error}", file=sys.stderr)
            failures += 1
            continue
        with tempfile.TemporaryDirectory(prefix="zend-opcodes-") as directory:
            entry = Path(directory) / fixture.with_suffix(".php").name
            for profile, settings in PROFILES.items():
                try:
                    output = check_fixture(php, settings, entry, source, expected)
                    if fixture == TESTS / "project/folded_branch.php":
                        has_branch = re.search(rb"^\s*\d+:\s+JMPZ\b", output, re.MULTILINE) is not None
                        if has_branch != (profile == "ordinary"):
                            raise ValueError("unexpected JMPZ in the constant-folding check")
                except (OSError, ValueError, subprocess.TimeoutExpired) as error:
                    print(f"FAIL {label} [{profile}]: {error}", file=sys.stderr)
                    failures += 1
    if failures:
        print(f"{failures} failed", file=sys.stderr)
        return 1
    print(f"{len(fixtures)} fixtures passed both checks under both compiler profiles.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
