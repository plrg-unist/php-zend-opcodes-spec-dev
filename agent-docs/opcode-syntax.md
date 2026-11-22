# Concrete syntax for opcode fixtures

The native extension prints one compilation unit in a line-oriented, assembly-like syntax. The Rust importer reads the same syntax for generated and hand-written fixtures. The following is a proposed spelling for the instruction part. [Compiled data to preserve](opcode-input.md)

## A function body

For `function choose($x) { if ($x) { return $x + 1; } return 0; }`, the pinned default optimizer produced the instructions, slots, jump target and literal references shown here. This excerpt shows the local names, literal pool and code.

```text
function "choose" {
    locals ["x"]
    lit[0] = int(1)
    lit[1] = int(0)

    0: CV0 = RECV 1
    1: JMPZ CV0, @4
    2: T1 = ADD CV0, lit[0]
    3: RETURN T1
    4: RETURN lit[1]
}
```

## Printing and parsing conventions

- Use one instruction per line. Read its body-local index, optional `result =`, mnemonic, then `op1` and `op2` in that order, separated by a comma. A trailing empty operand can be omitted. Use `_` if an empty first operand precedes a meaningful second operand. Keep `OP_DATA` at its own index.
- Use the engine’s `CV`, `T` and `V` slot notation. `@4` denotes instruction 4 in this body. A bare integer, such as the argument position in `RECV 1`, is an immediate. `lit[n]` references entry `n` in the ordered literal pool. Pool order matters because some handlers also read the entry following an operand’s literal. [Paired name literals](../../php-src/Zend/zend_compile.c#L633) · [VM lookup](../../php-src/Zend/zend_vm_def.h#L3704)
- Write literal values in full. For example, `array("answer" => int(42))` describes an ordered array with a byte-string key. Quote byte strings, escape quotes and backslashes, and use `\xHH` for non-printable bytes. Give floating-point values a spelling that round-trips exactly.
- Use named instruction fields after `|` for data such as `ext` and `line`. Give body metadata such as argument information, live ranges and exception tables their own declarations. Define those spellings as the corresponding features enter the spec.
- Parse the shared instruction grammar first, then validate the opcode’s operand roles and references. Report the body, text line and offending field for unsupported or invalid input.
