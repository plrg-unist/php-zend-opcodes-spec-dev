# Testing the opcode printer and loader

The first extension milestone covers native opcode printing and loading. Start with the [PHP source corpus](test-corpus.md) on the pinned engine. The [future spec comparisons and VM observation](differential-testing.md) are a separate milestone.

## Test script

Use one `scripts/test_extension.py` with `#!/usr/bin/env python3`, type hints, standard-library modules and minimal comments. Set its executable bit with `chmod +x` and run it directly with Python 3. Discover `*.phpt` recursively under `tests/` and standalone `*.php` under `tests/project/`, in sorted order. Run the whole corpus with fixed profiles and checks on each invocation.

Use standalone PHP source or the bytes of a PHPT's `--FILE--` section. Each fixture gets a Python `TemporaryDirectory` for its source and listings. Replacing the temporary source bytes with the listing preserves the logical path and verifies independent execution. For simple `TEST`/`FILE`/`EXPECT` PHPTs, check the reference against the bundled expectation. Mixed-stream PHPT expectations and additional sections need the upstream runner. Standalone fixtures can exercise both streams and nonzero status. [Test script](../scripts/test_extension.py) · [Local corpus](test-corpus.md)

## Two checks per PHP fixture

For each fixture, run the ordinary and default-optimized profiles. Each fixture/profile pair needs two fresh PHP processes. Keep both opcode listings in temporary files so program stdout and stderr can be compared independently. [Compiler profiles](fixture-compilation.md)

1. Compile the PHP source, have the printer write its finalized unit to file A, then execute the original unit. Capture stdout, stderr and exit status.
2. Load A into native Zend structures. Before execution, use the same printer to write the reconstructed main body, declarations and metadata to file B. Execute the loaded unit with the same inputs, runtime settings and logical filename. Loading consumes A independently of the original PHP source and bypasses further optimization.

Require successful printing and loading, byte-for-byte equality of A and B, identical stdout and stderr bytes on their respective streams, and equal exit status. Report the fixture, profile and failed comparison and return a failing status on a mismatch. Canonical listings use body-local indices, stable declaration order and a consistent logical source filename. Preserve instruction and literal-pool order.

For a fixture designed to observe a scalar or array return value, an ordinary PHP wrapper can `require` either entry and write `serialize($result)` to a separate result file. A top-level `return` value is discarded by the ordinary CLI. [CLI execution](../../php-src/main/main.c#L2666)

A round trip can pass if both sides omit the same information. Conversely, two programs can give the same result for an input while differing on an unexecuted branch. Give branching fixtures inputs that exercise each relevant path. In a source-side check on both profiles, two PHP programs printed `done`, wrote nothing to stderr and exited `0`. Their separately captured return arrays distinguished `['answer' => 41]` from `['answer' => 42]`.

## Independent assembly checks

The runner executes a hand-written listing, changes an integer literal, and checks the changed output. It also checks that a missing literal and an invalid jump target produce a diagnostic and a failing exit before the first observable instruction. [Hand-written listing](../tests/project/handwritten.zop)

The execution comparison covers runtime effects. Compiler-only diagnostics (such as a deprecation during parsing) occur in the source process. A fixture that reads its own file bytes also needs special treatment because the second process reads a listing at that path.
