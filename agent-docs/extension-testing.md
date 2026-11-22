# Testing the opcode printer and loader

These checks cover native opcode printing and loading on the pinned engine. The planned Cargo integration test reuses this corpus and PHP invocation. [Local corpus](test-corpus.md) · [AST milestone tests](opcode-ast.md#tests) · [Future spec comparisons and VM observation](differential-testing.md)

## Test script

The executable Python script recursively discovers every `.php`, `.phpt` and `.zop` file under `tests/` in sorted order. Each fixture runs in a fresh temporary directory for each profile. Keep expectations with the fixtures so the runner can use the same checks regardless of filename. [Test script](../scripts/test_extension.py)

Use standalone PHP source or the bytes of a PHPT's `--FILE--` section. Replacing the temporary source bytes with the listing preserves the logical path and verifies independent execution. For simple `TEST`/`FILE`/`EXPECT` PHPTs, check the reference against the bundled expectation. Mixed-stream PHPT expectations and additional sections need the upstream runner. Standalone fixtures can exercise both streams and nonzero status. [Local corpus](test-corpus.md)

## Two checks per PHP fixture

For each fixture, run the ordinary and default-optimized profiles. Each fixture/profile pair needs two fresh PHP processes. The runner sets `ZEND_OPCODES_TEST_PROFILE` in each child so a fixture can check a profile-specific expectation independently of PHP's configuration. Keep both opcode listings in temporary files so program stdout and stderr can be compared independently. [Compiler profiles](fixture-compilation.md)

1. Compile the PHP source, have the printer write its finalized unit to file A, then execute the original unit. Capture stdout, stderr and exit status.
2. Load A into native Zend structures. Before execution, use the same printer to write the reconstructed main body, declarations and metadata to file B. Execute the loaded unit with the same inputs, runtime settings and logical filename. Loading consumes A independently of the original PHP source and bypasses further optimization.

Require successful printing and loading, byte-for-byte equality of A and B, identical stdout and stderr bytes on their respective streams, and equal exit status. Report the fixture, profile and failed comparison and return a failing status on a mismatch. Canonical listings use body-local indices, stable declaration order and a consistent logical source filename. Preserve instruction and literal-pool order.

For a fixture designed to observe a scalar or array return value, an ordinary PHP wrapper can `require` either entry and write `serialize($result)` to a separate result file. A top-level `return` value is discarded by the ordinary CLI. [CLI execution](../../php-src/main/main.c#L2666)

A round trip can pass if both sides omit the same information. Conversely, two programs can give the same result for an input while differing on an unexecuted branch. Give branching fixtures inputs that exercise each relevant path. In a source-side check on both profiles, two PHP programs printed `done`, wrote nothing to stderr and exited `0`. Their separately captured return arrays distinguished `['answer' => 41]` from `['answer' => 42]`.

## Hand-written listings

Load a `.zop` fixture directly, then reload its canonical printout. Compare the two printouts, stdout, stderr and exit status. The input may use any whitespace the loader accepts.

Use a companion file when a listing needs an independent expectation. For a successful program, `name.zop.out` gives its exact stdout bytes, with empty stderr and exit status zero. For an intentionally invalid program, `name.zop.error` gives a diagnostic excerpt expected on stderr, with a failing exit, empty stdout and no emitted listing. Each listing accepts at most one companion. [Hand-written listing](../tests/project/handwritten.zop) · [Rejection fixtures](../tests/project/invalid)

The execution comparison covers runtime effects. Compiler-only diagnostics (such as a deprecation during parsing) occur in the source process. A fixture that reads its own file bytes also needs special treatment because the second process reads a listing at that path.
