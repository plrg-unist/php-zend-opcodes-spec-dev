# Execution abstraction review

This review covers the implementation through milestone 3D at `36665e6` and the pinned PHP 8.6.0 CLI. Two earlier decisions provide the baseline. At that point the spec abstracted automatic cycle collection timing and received 16 installation constant values as inputs. [Cycle collection decision](../design-docs/automatic-cycle-collection.md) · [Execution inputs](../design-docs/execution-io.md) · [PHP build and compiler profiles](fixture-compilation.md)

Each **Open** entry contains a review recommendation awaiting the user's decision. Update the entry in place as it is discussed and link the resulting design record where applicable. [How observable differences are decided](semantics-scope.md#choosing-what-to-model)

## 1. Runtime constant names and values

**Implemented.** Predefined names and their typed values come from execution input by default. Of the 552 names at the pinned CLI, the checked-in reference file supplies 549. Sixteen already received installation values, and this revision moved the other 533 ordinary values out of the rules. A program declaration enters request state when executed. A literal folded during compilation carries its value in the unit. [Input contract and CLI](../design-docs/execution-io.md#runtime-constants) · [Reference data](../runtime-constants.json) · [Runtime lookup and declaration](../spec/02-vm/21-constants.watsup)

The three case-insensitive global names `TRUE`, `FALSE` and `NULL` belong to the core. PHP uses the first two to spell its two Boolean values and the third to spell the sole value of the null type. Zend recognizes these three before ordinary constant lookup during compilation, and its runtime name resolution also recognizes mixed-case spellings. On the pinned CLI, `defined('NuLl')` returned `true` and `constant('NuLl')` returned `null`. [PHP Boolean literals](https://www.php.net/manual/en/language.types.boolean.php) · [PHP null literal](https://www.php.net/manual/en/language.types.null.php) · [Compiler substitution](../../php-src/Zend/zend_compile.c#L1703) · [Runtime lookup](../../php-src/Zend/zend_constants.c#L222)

The differential suite checks `GLOB_MARK` against explicit expected values of `8` and `2` on the same finalized listing. It also exercises a missing name, a defined `null`, a long namespaced name, tagged values, deprecation messages and a folded literal. [Per-run test inputs](differential-testing.md) · [Regression fixture](../tests/project/runtime_constant_inputs.php) · [Replacement table](../tests/project/runtime_constants_subset.json)

The following probes used runtime lookups so that each comparison executed the same finalized listing. All runs had empty stderr and exit status 0.

| Lookup observed by the program | Reference stdout | Changed condition | Other stdout |
| --- | --- | --- | --- |
| `namespace Audit; echo GLOB_MARK;` | `8` | Same PHP source and 64-bit target, built with system glob | `2` |
| Test `defined('PHP_BUILD_PROVIDER')`, then print its value or `absent` | `absent` | Same PHP source, configured with a build-provider string | `ReviewBuild` |
| Print `(int) PHP_CLI_PROCESS_TITLE` in a namespace | `1` | Same PHP executable launched through the Linux dynamic loader with `--argv0 php` | `0` |

PCRE version and JIT support, debug and thread-safety flags, VM dispatch kind and the configured PHP version suffix also depend on the build. [Reference catalog](../runtime-constants.json) · [Glob flag definitions](../../php-src/main/php_glob.h#L59) · [Optional provider](../../php-src/main/main_arginfo.h#L15) · [CLI capability check](../../php-src/sapi/cli/php_cli.c#L920)

### Comparison with the Python bytecode spec

At `python-spectec` revision `d595687`, CPython initializes the process and compiles the program before the spec takes over frame evaluation. Its bridge imports the live `sys` module, translates its dictionary and the interpreter's builtins dictionary, and obtains each frame's own builtins mapping. The opcode rules perform ordinary lookup against those objects. Consequently, `sys.maxsize` comes from the host build, `sys.flags.optimize` from the launch, and names such as the Windows-only `sys.winver` can be absent. The corpus exercises both importing `sys.maxsize` and supplying a custom builtins mapping. [Startup bridge](../../python-spectec/python/target/engine/bootstrap.ml#L338) · [Frame inputs](../../python-spectec/python/target/types/frame_type.ml#L108) · [Attribute lookup](../../python-spectec/python-spec/6-eval/6.4.10-bytecode-attribute.spectec#L14) · [Import fixture](../../python-spectec/python/testdata/main/import_star.py#L6) · [Custom mapping fixture](../../python-spectec/python/testdata/main/builtins_mapping.py#L14) · [CPython system parameters](https://docs.python.org/3.14/library/sys.html#sys.maxsize)

The Python spec gives distinguished identities to `None`, `True`, `False` and `NotImplemented`. Its `LOAD_CONST` rule reads the code object's constant pool. CPython folds a bare `__debug__` using the compilation's optimization level. A probe compiled once at optimization level 0 printed `__debug__`, `builtins.__debug__` and `hasattr(builtins, "help")`. On stock CPython 3.12.3 it printed `True True True` normally, `True False True` with `-O`, and `True True False` with `-S`. The Python 3.14 project behavior was checked against its source and fixtures. [Singleton identities](../../python-spectec/python-spec/2-memory/2.1-memory.spectec#L75) · [Constant-load rule](../../python-spectec/python-spec/6-eval/6.4.13-bytecode-constant.spectec#L46) · [CPython compiler](https://github.com/python/cpython/blob/3.14/Python/ast_preprocess.c#L589) · [Builtins added by site](https://docs.python.org/3.14/builtins/constants.html#constants-added-by-the-site-module)

The distinction matters even within one project. A program reading `sys.maxsize` gets the host's value, yet the Python spec's sequence-repetition rules fix an internal limit of `2^63 - 1` and assume eight-byte pointers. For PHP, deciding where a public constant gets its value and justifying any target assumption in an operation are separate questions. [Sequence size limits](../../python-spectec/python-spec/5-builtin/5.06-builtin-list.spectec#L180)

## 2. Floating-point power

With only the script path on the command line, `$argc` is 1. The probe used `$x = 2.4 * $argc; echo (int) ($x ** 11.1 === 16609.04963655714);` to compute the power at runtime. Both Zend and the spec printed `1` with default glibc CPU dispatch and `0` with `GLIBC_TUNABLES=glibc.cpu.hwcaps=-FMA,-FMA4,-AVX2,-AVX,-AVX512F`. Each compiler profile reproduced the change on its own listing, with empty stderr and exit status 0.

**Open.** State that the target math library supplies the final rounding of floating-point `POW`. PHP's operand coercions, result type, diagnostics and integer overflow promotion still have independent rules. The current Rust primitive already delegates to `powf`. [Power rules](../spec/01-values/03-arithmetic.watsup) · [Zend's call to pow](../../php-src/Zend/zend_operators.c#L1415) · [Rust primitive](../src/binary64.rs#L68) · [Rust precision contract](https://doc.rust-lang.org/std/primitive.f64.html#method.powf)

## 3. Native opcode layout and listing preparation

Switch and match instructions branch to body-local instruction indices. A hand-written listing can also read its jump table as an ordinary PHP array. The existing fixture prints the native byte offsets `32,-32,0\n`. A C probe using Zend's offset macro on the same 64-bit headers produced `56,-56,0` when compiled with `ZEND_VERIFY_TYPE_INFERENCE`. That result measures the alternate structure layout. The PHP execution used the reference build. [Fixture](../tests/project/array_dispatch_offsets.zop) · [Spec's 32-byte multiplier](../spec/02-vm/02-cases.watsup#L12) · [Zend opcode structure](../../php-src/Zend/zend_compile.h#L137) · [Offset macro](../../php-src/Zend/zend_compile.h#L765)

**Open.** Keep branch targets as indices and obtain any exposed byte offsets from listing preparation for the target build. Two related preparation steps can supply their results to the VM while preserving observable behavior.

- The pinned loader reprints `MUL lit[1], CV0` as `MUL CV0, lit[1]`. The resulting type error says `int * string`. Execution can consume that finalized operand order. [Listing](../tests/project/numeric_multiply_operands.zop) · [Operand ranking in the VM](../spec/02-vm/00-core.watsup#L260)
- Preparation can identify which literal arrays are immutable. Changing the mask of a dormant partial call from 1 to 0 changed earlier stdout from `\n` to `acefghxy\n` in both executors. The VM currently scans call sites to recover that property. [Dormant call fixture](../tests/project/foreach_cursor_dormant_partial.zop#L51) · [Preparation rules](../spec/02-vm/17-literal-preparation.watsup#L101)

## 4. Tool nesting limits and well-formedness

A program with 65 nested closure defaults in an unused function printed `ok` on pinned Zend under both compiler profiles. The exporter wrote both listings successfully. The spec classified each as `malformed` with `constant closure nesting exceeds 64`. At depth 64 the spec also printed `ok`. In a separate probe Zend ran an unused default containing 257 additions around an unresolved constant. The exporter rejected the expression, and the Rust validator classified a listing with that expression as malformed. [Closure limit](../src/validate/expressions.rs#L117) · [Expression limit](../src/validate/expressions.rs#L54) · [Native expression codec](../ext/constant_ast.c#L72)

**Open.** Treat these cutoffs as capacities of the tools. A valid unit that exceeds a traversal limit should either execute or receive a tool-level failure. Reserve `malformed` for a structural defect in the input. [Outcome definitions](../design-docs/execution-io.md)

## 5. Array capacity and the next append key

Two PHP literals produce the same entries at keys 0 through 3. Repeating a key in the first literal reserves more storage. Both arrays then write and unset key 12, write key 4, and append. In both compiler profiles Zend and the spec append at key **5** for the wider array and **13** for the smaller one. The fixture prints `9:-\n-:9\n`, with empty stderr and status 0. Zend introduced the packed-array update in a performance optimization. Matching it requires the spec to track capacity and when sparse writes change the table representation. [PHP fixture](../tests/project/array_literal_packed_capacity.phpt) · [Spec updates](../spec/01-values/10-array-updates.watsup#L28) · [Zend optimization](https://github.com/php/php-src/commit/f4b7239caeb56d41d4458890b5e7f1f171c7cf6b)

**Open.** The review recommends retaining the highest assigned integer key across these ordinary writes and deletions, as described in the PHP manual. Both appends would use key 13 and the fixture would print `-:9\n-:9\n`. This would be a deliberate difference from pinned Zend. Native replay can retain exact layout metadata. Iteration order, cursor behavior and reference effects need their own witnesses when considering any further simplification. [PHP array append rule](https://www.php.net/manual/en/language.types.array.php#language.types.array.syntax.modifying) · [Physical cursor fixture](../tests/project/cursor_layout_rebind.zop)

## 6. Array sharing and comparison

With `$x = [$argc * NAN]; $y = $x;`, the comparison `$x === $y` is true. After `unset($y['absent'])`, it is false. Unsetting the missing key leaves every entry in place and causes Zend to separate the backing tables. Printing the two comparison results as integers produced `10` in Zend and the spec under both profiles, with empty stderr and status 0. Zend introduced the same-table shortcut as a comparison optimization. [Spec identity check](../spec/01-values/08-array-comparison.watsup#L97) · [Missing-key unset fixture](../tests/project/copy_on_write_noop.php) · [Zend optimization](https://github.com/php/php-src/commit/d57b278ad1f490094b0c3331cf02312b785a8a78)

**Open.** The review favors retaining the pinned result for now. An entry-wise comparison of these acyclic arrays would print `00`, since `NAN` compares unequal to itself. This choice is independent of the append key. A recursive array comparing identical to itself is also covered by an upstream regression. [PHP array comparisons](https://www.php.net/manual/en/language.operators.array.php) · [Recursive comparison regression](../../php-src/Zend/tests/recursive_array_comparison.phpt)

## 7. Decimal conversions

- **Exponent cutoff.** A string consisting of `1`, 20,000 zeroes and `e-20000` represents exactly one. Casting it to float and printing it produced `10` in Zend and the spec under both profiles. Zend caps the explicit exponent at ±19,999 before accounting for the mantissa, a precaution for 16-bit integers. Decimal conversion without that cutoff would print `1`. [Zend cutoff](../../php-src/Zend/zend_strtod.c#L2695) · [Rust parser](../src/binary64.rs#L114)
- **Incomplete signed exponent.** `"9300000000000000000e+" + 0` emits a warning and yields the integer `-9146744073709551616`. Zend advances the scan position past the `e` before checking the 19-digit prefix for overflow. Applying the documented leading-numeric-string rule would preserve the warning and yield a positive float. [Spec range check](../spec/01-values/02-numeric-conversions.watsup#L88) · [Zend scanner](../../php-src/Zend/zend_operators.c#L3743) · [Fixture](../tests/project/numeric_decimal_limits.php) · [PHP numeric strings](https://www.php.net/manual/en/language.types.numeric-strings.php)

**Open.** Both observations were repeatable on the pinned target. Changing either conversion is a separate compatibility decision with an observable result.

## Other effects checked

- Live reference ownership changes later writes. With `$x = 1; $a = [&$x]; $b = $a; $b[0] = 7;`, printing the two array elements gives `77`. Inserting `unset($x)` before the copy gives `17`. Keep the aliasing effect. [Reference ownership rules](../spec/02-vm/14-reference-arrays.watsup) · [Regression fixture](../tests/project/refs_copy_owners.phpt)
- In the namespace fallback fixture the ordinary compiler assigns separate constant-cache slots to two fetches, while the optimized listing shares one slot. They print `global|global|local\n` and `global|global|global\n` respectively. The finalized listing supplies the slot assignments. [Cache rules](../spec/02-vm/21-constants.watsup#L145) · [Namespace fixture](../tests/project/review_constant_namespace_cache.php)
- Signed 64-bit integers and the fixed initial CLI diagnostic settings fit the agreed target. The three standard streams receive resource IDs 1, 2 and 3 in both reference profiles. A probe that opened and closed one resource in an additional startup extension shifted the IDs to 2, 3 and 4. Stream numbering would need revisiting if the startup profile were broadened. [Execution policy](semantics-scope.md#execution-policy) · [CLI stream creation](../../php-src/sapi/cli/php_cli.c#L512) · [Resource ID fixture](../tests/project/standard_stream_values.phpt)
