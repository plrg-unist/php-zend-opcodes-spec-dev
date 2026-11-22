# PHP source corpus

Keep all test inputs in the project's `tests/` directory. Use the local PHPT copies as inputs and generate opcode variants during testing. [Fixture provenance and license](../tests/README.md) · [Compiler profiles](fixture-compilation.md) · [Printer and loader checks](extension-testing.md)

## Starter selection

Each case uses only `--TEST--`, `--FILE--` and `--EXPECT--`, compiles as one unit, and produces deterministic CLI output. The paths below are relative to `tests/php-src/`.

| Behavior | Local fixture |
| --- | --- |
| Literal expression and output | [tests/func/001.phpt](../tests/php-src/tests/func/001.phpt) |
| Conditional branch | [tests/lang/001.phpt](../tests/php-src/tests/lang/001.phpt) |
| Loop with a back edge | [tests/lang/002.phpt](../tests/php-src/tests/lang/002.phpt) |
| Passing arguments by value and reference | [tests/lang/passByReference_001.phpt](../tests/php-src/tests/lang/passByReference_001.phpt) |
| Function-local statics across calls | [tests/func/002.phpt](../tests/php-src/tests/func/002.phpt) |
| Array append and copy-on-write | [Zend/tests/array_append_COW.phpt](../tests/php-src/Zend/tests/array_append_COW.phpt) |
| Foreach by value and reference | [tests/lang/foreach_with_references_001.phpt](../tests/php-src/tests/lang/foreach_with_references_001.phpt) |
| Class, magic methods and property updates | [tests/classes/assign_op_property_001.phpt](../tests/php-src/tests/classes/assign_op_property_001.phpt) |
| Integer match table | [Zend/tests/match/001.phpt](../tests/php-src/Zend/tests/match/001.phpt) |
| Nested closures and captured variables | [Zend/tests/arrow_functions/001.phpt](../tests/php-src/Zend/tests/arrow_functions/001.phpt) |
| Throw, catch and nested finally blocks | [Zend/tests/try/try_catch_finally_001.phpt](../tests/php-src/Zend/tests/try/try_catch_finally_001.phpt) |
| Generator suspension and delegation | [Zend/tests/generators/yield_from_array.phpt](../tests/php-src/Zend/tests/generators/yield_from_array.phpt) |

All 12 passed php-src's `run-tests.php` under both the ordinary and default-optimized profiles on the pinned Linux CLI. A compile-time probe observed 55 distinct opcode names across the two profiles. Five fixtures changed their opcode sequence between profiles.

Pass the local `tests/php-src/` directory to php-src's `run-tests.php` to check the fixtures against their bundled expectations. For the extension tests, write the `--FILE--` section to a named `.php` file. Keep the same logical filename for the source and loaded runs. Compare their actual stdout, stderr and status directly. PHPT expectations describe output under the upstream runner's own matching rules. [Running PHPTs](../../php-src/docs/source/miscellaneous/running-tests.rst) · [Expected-output matching](../../php-src/run-tests.php#L2863)

## Growing the corpus

- The pin contains 23,731 tracked PHPTs. Search `Zend/tests/`, `tests/lang/`, `tests/classes/` and `tests/func/` first. Of their 6,314 tests, 2,882 have just the three sections used by the starter selection.
- Copy each selected PHPT and any helper inputs into `tests/php-src/` at their upstream paths. Preserve any additional license notices and record exceptions to the shared revision or license in the [test directory note](../tests/README.md).
- Prefer single-unit cases with reproducible CLI behavior. A compile-error test yields no finalized unit. Honor `SKIPIF`, `INI`, `EXTENSIONS`, external inputs and cleanup when adding tests that depend on them. A source that compiles but needs extra setup to execute can still exercise the structural round trip if labeled compile-only. [PHPT sections](../../php-src/docs/source/miscellaneous/writing-tests.rst)
- For an optimizer-specific gap, inspect `ext/opcache/tests/opt/`. Some expectations there are diagnostic opcode dumps produced by test-specific settings. Reuse the source under the project profiles and supply a runtime check when the fixture only declares a function. [Upstream example](../../php-src/ext/opcache/tests/opt/dce_008.phpt)
- Put cases written for this project in `tests/project/`. Create the directory with the first fixture, for example one that exercises a missing instruction, operand form or regression.
