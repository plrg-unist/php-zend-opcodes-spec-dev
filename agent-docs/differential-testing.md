# Differential testing against the Zend Engine

The [printer and loader tests](extension-testing.md) exercise the full codec corpus. The [semantics suite](../scripts/test_semantics.py) compares executions within the [agreed runtime scope](semantics-scope.md).

## Spec executable and differential suite

- `zend-opcodes [--json] FILE.zop` reads the program from the path and PHP input from process stdin. In normal mode forward program stdout and stderr to their respective streams and return its process status, including for PHP failures. Panic at the CLI boundary for non-PHP outcomes in this mode. Keep the evaluator's outcomes explicit.
- In JSON mode emit one object on stdout and exit 0 for all reported outcomes. For `kind: "php"` include `stdout` and `stderr` as ordered arrays of integers from 0 to 255 and `exit_status` as the PHP process status. Python can recover each stream with `bytes(values)`. Use distinct kinds for `unsupported`, `malformed`, and `tool_error`. Preserve bytes already emitted before an unsupported operation. An evaluator defect or failed invocation fails the test.
- Run the standalone `./scripts/test_semantics.py`. It builds the executable once with `cargo build --bin zend-opcodes --message-format=json` and takes its path from Cargo's binary artifact. It reuses [the PHP fixture helper](../scripts/zend_fixtures.py). Export each PHP or PHPT source once per [compiler profile](fixture-compilation.md) with `php -l` in a temporary directory. Hand-written fixtures already supply the listing. For each selected input, run Zend's loader and the spec executable in fresh processes on the same listing with identical stdin bytes followed by EOF. Compare stdout bytes, stderr bytes and exit status. Exercise normal-mode stream forwarding as well as JSON mode. [Cargo build messages](https://doc.rust-lang.org/cargo/reference/external-tools.html#json-messages)
- The [manifest](../tests/semantics.json) lists required executions. Each row names a `fixture` and optionally a raw `stdin` file, both relative to `tests/`. Omitted stdin means empty bytes followed by EOF. Store input bytes under `tests/stdin/<fixture-name>/`. Multiple rows can select different inputs for the same fixture. The default expectation is `php`. For `expect` set to `unsupported`, optional `stdout` and `stderr` byte arrays specify the output emitted before stopping. Both default to empty. A `malformed` row must fail before execution and produce no program output or coverage report. Only `php` rows run the native loader for this comparison. The codec suites keep discovering all program fixtures.
- For a later-milestone feature, use a focused temporary `unsupported` expectation that reaches that operation through supported setup. An unexpected PHP result also fails this expectation. When the feature lands, normally keep the input, switch its expectation to `php`, and check its observable result against Zend. Revise or retire a redundant transitional case. Keep boundary cases for features permanently outside scope.
- Add support for an optional `divergence` field on `php` manifest rows when the first reproducible semantic difference is approved. Each exceptional row links to the rule's rationale in [design-docs](../design-docs/README.md). Run both Zend and the spec. For each field that deliberately differs, check both sides against their own exact expected values. Compare every other field directly with Zend.

At file scope, `return 7;` and `return "7";` both finish with empty output and exit status 0. Check file-level return through its control-flow effect, such as skipping a later write. For a function result, make the calling fixture expose what matters. `exit(answer());` works for a small integer, and `exit(answer() === 7 ? 0 : 1);` checks both value and type. The initial suite compares only the three CLI observables. [CLI return handling](../../php-src/main/main.c#L2666)

## Spec coverage

The standalone script enforces 100% cumulative structured-language (SL) instruction coverage. For each instrumented run it sets `ZEND_SPEC_COVERAGE` to a temporary report path. The executable attaches P4-SpecTec's `InstrHandler`, disables interpreter tail-call optimization so result instructions remain visible, and writes P4-SpecTec's per-step report separately from program output. The script checks the report totals and the ordered step identities before combining passing cases. [Coverage implementation](../../p4-spectec/p4spectec/core/src/coverage/instr.rs) · [Execution host](../src/main.rs)

Combine hit IDs from cases that met their expectations across fixtures, stdin variants and both compiler profiles. Pass only if all required tests pass and the union covers every numbered executable instruction in the current production spec, with a nonzero total. Use exact counts, include helper definitions and apply no coverage-ignore hints. Print the owning definition and step for any miss. If an invariant or lowering makes a counted step unreachable, investigate its purpose, simplify redundant rules when correct, and report any unresolved gap before declaring the milestone complete. [Report format](../../p4-spectec/p4spectec/core/src/coverage/log.rs)

At milestone review, check the [promised behaviors](semantics-roadmap.md) independently of the hit count. Identify what output or status would change for an incorrect result, which relevant operand and control-flow combinations the fixtures exercise, and which Zend behavior might lack a spec rule altogether.

The SL coverage report counts the spec instructions that call Rust primitives. Check each primitive's implementation separately with `cargo test` and with end-to-end cases against Zend. For binary64, exercise both sides of a rounding boundary and verify the printed bytes under ordinary output and diagnostics. [Rust numeric primitives and tests](../src/binary64.rs)

## Future VM observations

With the ordinary `php` executable alone, a fixture can explicitly print values or call `get_defined_vars()` in the scope of interest. Automatic VM checkpoints in an unmodified fixture need instrumentation. The existing extension could host an observer. For interactive inspection, the separate `phpdbg` executable supports `set stepping opcode` and `info vars`. [Scope variables](../../php-src/Zend/zend_builtin_functions.c#L1506) · [Debugger stepping](../../php-src/sapi/phpdbg/phpdbg_set.c#L216) · [Debugger variables](../../php-src/sapi/phpdbg/phpdbg_info.c#L36)

- **Returns and exceptions.** Zend’s call observer can receive a file or function return value while the frame is live. The error observer supplies diagnostic type, file, line and message. This permits differential checks of typed returns and exceptional completion when those observations are needed. [Return observation](../../php-src/Zend/zend_vm_def.h#L4546) · [Call observer](../../php-src/Zend/zend_observer.h#L49) · [Error observer](../../php-src/Zend/zend_observer.h#L152)
- **State checkpoints.** A user-opcode handler runs before the selected instruction. Record its body and index, read selected live locals or operands from `zend_execute_data`, and return `ZEND_USER_OPCODE_DISPATCH` to continue ordinary execution. Observe effects at the next appropriate checkpoint or frame exit, while the relevant values are live. Selected globals are available in `EG(symbol_table)`. For reachable state, compare PHP types, ordered array entries, stored object properties and object/reference identity using logical IDs. Allocator layout and addresses are process-specific. [Frame layout](../../php-src/Zend/zend_compile.h#L646) · [Opcode hooks and operand access](../../php-src/Zend/zend_execute.c#L5965) · [Global symbol table](../../php-src/sapi/phpdbg/phpdbg_info.c#L185) · [References](../../php-src/Zend/zend_types.h#L633)
- **Trace alignment.** Compare checkpoints within the same captured profile. OPcache can eliminate locals and instructions, and enabling a call observer can change frame sizing. Export the program during the observed run. Some handlers consume companion rows such as `OP_DATA`, so one dispatch can cover multiple instruction rows. [Observer frame slot](../../php-src/Zend/zend_opcode.c#L1146) · [Array assignment](../../php-src/Zend/zend_vm_def.h#L2680)

## Verified on the pinned engine

A temporary regular extension observed the same main and `bump` bodies that its compiler hook captured for this fixture.

```php
<?php
function bump(&$x) {
    $alias =& $x;
    $alias += 2;
    return $alias;
}
$n = 40;
$r = bump($n);
return [$n, $r];
```

With OPcache off and with its default optimizer, both output streams were empty and the exit status was `0`. Just before `ASSIGN_OP` in `bump`, `$x` and `$alias` referred to the same reference containing `int(40)`. Just before `RETURN` they still shared that reference, now containing `int(42)`. The call observer saw `int(42)` returned by `bump` and the two-element integer array `[42, 42]` returned by the file. A separate fixture produced an observed `E_USER_WARNING`, an escaping `RuntimeException` and exit status `255` in both profiles.
