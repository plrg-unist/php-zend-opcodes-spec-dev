#!/usr/bin/env python3
import subprocess
import sys
import tempfile
from pathlib import Path

sys.dont_write_bytecode = True
from zend_fixtures import (
    PROFILES, ROOT, check_expected, compare_execution, corpus, execute, expected_error, read_fixture,
)


def check_fixture(php: Path, profile: str, fixture: Path, directory: Path) -> None:
    source, expected = read_fixture(fixture)
    rejection = expected_error(fixture)
    reference = execute(php, profile, fixture, directory, source, rejection=rejection)
    if rejection is None:
        check_expected(fixture, reference, expected)
        candidate = execute(php, profile, fixture, directory, reference.listing, load=True)
        compare_execution(reference, candidate)


def main() -> int:
    php, fixtures = corpus()
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
    try:
        sys.exit(main())
    except (OSError, ValueError) as error:
        sys.exit(str(error))
