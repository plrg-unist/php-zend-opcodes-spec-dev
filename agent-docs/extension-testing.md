# Testing the opcode printer and loader

The first extension milestone covers native opcode printing and loading. Start with the [PHP source corpus](test-corpus.md) on the pinned engine. The [future spec comparisons and VM observation](differential-testing.md) are a separate milestone.

## Test script

Use one `scripts/test_extension.py` with `#!/usr/bin/env python3`, type hints, standard-library modules and minimal comments. Set its executable bit with `chmod +x` and run it directly with Python 3. Discover `*.phpt` recursively under `tests/` and standalone `*.php` under `tests/project/`, in sorted order. Run the whole corpus with fixed profiles and checks on each invocation.

Use standalone PHP source or the bytes of a PHPT's `--FILE--` section. Each fixture gets a Python `TemporaryDirectory` for its source and listings. The same temporary script path is used for both runs. Replacing its source bytes with the listing ensures the loader can execute independently. The current PHPTs use only `TEST`, `FILE` and `EXPECT` and expect successful stdout-only execution. Check that reference against its bundled expectation, then compare the loaded run's streams as raw bytes. Report unsupported sections with their fixture path. [Local corpus](test-corpus.md)

## Two checks per PHP fixture

For each fixture, run the ordinary and default-optimized profiles. Each fixture/profile pair needs two fresh PHP processes. Keep both opcode listings in temporary files so program stdout and stderr can be compared independently. [Compiler profiles](fixture-compilation.md)

1. Compile the PHP source, have the printer write its finalized unit to file A, then execute the original unit. Capture stdout, stderr and exit status.
2. Load A into native Zend structures. Before execution, use the same printer to write the reconstructed main body, declarations and metadata to file B. Execute the loaded unit with the same inputs, runtime settings and logical filename. Loading consumes A independently of the original PHP source and bypasses further optimization.

Require successful printing and loading, byte-for-byte equality of A and B, identical stdout and stderr bytes on their respective streams, and equal exit status. Report the fixture, profile and failed comparison and return a failing status on a mismatch. Canonical listings use body-local indices, stable declaration order and a consistent logical source filename. Preserve instruction and literal-pool order.

For a fixture designed to observe a scalar or array return value, an ordinary PHP wrapper can `require` either entry and write `serialize($result)` to a separate result file. A top-level `return` value is discarded by the ordinary CLI. [CLI execution](../../php-src/main/main.c#L2666)

A round trip can pass if both sides omit the same information. Conversely, two programs can give the same result for an input while differing on an unexecuted branch. Give branching fixtures inputs that exercise each relevant path. In a source-side check on both profiles, two PHP programs printed `done`, wrote nothing to stderr and exited `0`. Their separately captured return arrays distinguished `['answer' => 41]` from `['answer' => 42]`.

## Later additions for the opcode syntax

- Keep a small hand-written [opcode fixture](opcode-syntax.md) with a known result and a canonical expected listing. Where there is a matching PHP source, check its printed output against the hand-written expectation as well. Editing one literal in the hand-written fixture and checking the changed result directly exercises the loader.
- Include a small rejection case, such as a reference to a missing literal. Expect an error identifying the relevant body and operand, a failing exit status, and no fixture execution.
