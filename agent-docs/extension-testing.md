# Testing the opcode printer and loader

The first extension milestone covers native opcode printing and loading. Start with the [PHP source corpus](test-corpus.md) on the pinned engine. The [future spec comparisons and VM observation](differential-testing.md) are a separate milestone.

## Two checks per PHP fixture

1. **Structural round trip.** Compile the PHP fixture and print its finalized unit to canonical text A. In a fresh PHP process, load A into native Zend structures, then use the same printer to print the reconstructed main body, declarations and metadata as text B. Require successful completion and a byte-for-byte `A == B`. Make output deterministic with body-local indices, stable declaration order and a consistent logical source filename. Preserve instruction and literal-pool order. The loaded run should consume the exported fixture independently of the original PHP file.
2. **Execution comparison.** Execute the original PHP fixture as the reference and the loaded program as the candidate in separate processes. Supply the same inputs, runtime settings and logical paths. Compare stdout and stderr as bytes and compare exit status. Make the effect under test observable. For fixtures returning scalars or arrays, an ordinary PHP wrapper can `require` either entry and write `serialize($result)` to a separate result file. This preserves distinctions such as integer `42` versus string `"42"` and leaves program output available for comparison. A top-level `return` value is discarded by the ordinary CLI. [CLI execution](../../php-src/main/main.c#L2666)

Run both checks for the ordinary and default-optimized [compiler profiles](fixture-compilation.md). Each exported variant is already finalized. The loader should send that variant directly to Zend and the printer. A second optimization pass could change the subject of the comparison.

A round trip can pass if both sides omit the same information. Conversely, two programs can give the same result for an input while differing on an unexecuted branch. Give branching fixtures inputs that exercise each relevant path. In a source-side check on both profiles, two PHP programs printed `done`, wrote nothing to stderr and exited `0`. Their separately captured return arrays distinguished `['answer' => 41]` from `['answer' => 42]`.

## Later additions for the opcode syntax

- Keep a small hand-written [opcode fixture](opcode-syntax.md) with a known result and a canonical expected listing. Where there is a matching PHP source, check its printed output against the hand-written expectation as well. Editing one literal in the hand-written fixture and checking the changed result directly exercises the loader.
- Include a small rejection case, such as a reference to a missing literal. Expect an error identifying the relevant body and operand, a failing exit status, and no fixture execution.
