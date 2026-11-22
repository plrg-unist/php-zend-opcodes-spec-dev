# Differential testing against the Zend Engine

This note covers future spec comparisons and VM observation. The first extension milestone is the [opcode printer and loader](extension-testing.md).

## Run the same program

For each PHP fixture and [compiler profile](fixture-compilation.md), make an ordinary CLI run with the exporter enabled. The compiler hook writes the finalized unit and returns the original main `zend_op_array`. Zend then passes that array to `zend_execute` in the same request. Run the spec on the exported [opcode text](opcode-syntax.md) with the same fixture inputs, including stdin, arguments and any environment or files the fixture uses. Keep the export and observation records on channels separate from the PHP program’s stdout and stderr. `php -l` remains appropriate for export-only runs. [Compile and execute path](../../php-src/Zend/zend.c#L1981) · [CLI execute and lint modes](../../php-src/sapi/cli/php_cli.c#L936)

The stock CLI accepts PHP source. OPcache can reuse a compiled unit through its own cache format. The planned loader in the regular PHP extension will also accept hand-written opcode fixtures. For a recognized fixture, its compiler hook validates the parsed unit, constructs the finalized `zend_op_array` bodies and their literals, jumps, frame layout and declarations, and returns the main body to Zend for normal execution. A supported hand-written fixture can then be supplied directly to both Zend and the spec. [Compiler hook](../../php-src/Zend/zend_compile.h#L895) · [Native execution API](../../php-src/Zend/zend_execute.h#L47) · [OPcache file-cache loader](../../php-src/ext/opcache/zend_file_cache.c#L1918)

## What to compare

With the ordinary `php` executable alone, a fixture can explicitly print values or call `get_defined_vars()` in the scope of interest. Automatic VM checkpoints in an unmodified fixture need instrumentation. The proposed exporter can also act as the observer. For interactive inspection, the separate `phpdbg` executable supports `set stepping opcode` and `info vars`. [Scope variables](../../php-src/Zend/zend_builtin_functions.c#L1506) · [Debugger stepping](../../php-src/sapi/phpdbg/phpdbg_set.c#L216) · [Debugger variables](../../php-src/sapi/phpdbg/phpdbg_info.c#L36)

- **Routine comparisons.** Capture stdout and stderr as bytes, process exit status, normal return versus an escaping exception, and the typed file return value on normal completion. Fixtures can return selected final values in an array. The standard CLI discards its main script’s return value. Zend’s call observer can receive a file or function return value while the frame is live. The error observer supplies diagnostic type, file, line and message. Distinguish a caught exception from the outcome at the file boundary. [CLI return handling](../../php-src/main/main.c#L2666) · [Return observation](../../php-src/Zend/zend_vm_def.h#L4546) · [Call observer](../../php-src/Zend/zend_observer.h#L49) · [Error observer](../../php-src/Zend/zend_observer.h#L152)
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
