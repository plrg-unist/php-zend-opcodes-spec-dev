# Foreign calls in the opcode spec

## Functions supplied by this project

The `.watsup` rules call five pure `extern` functions implemented in Rust. They pass each PHP float as its 64-bit IEEE 754 encoding. The rules choose when PHP converts a value and which warnings or errors it produces. [Declarations](../spec/01-values/00-binary64.watsup) · [Rust implementation](../src/binary64.rs)

| Function | What Rust computes |
| --- | --- |
| `$f64_from_int` | Rounds a signed 64-bit integer to binary64. |
| `$f64_binary` | Computes binary64 `ADD`, `SUB`, `MUL`, `DIV`, or `POW` on two floats. |
| `$f64_trunc` | Truncates a finite float toward zero and returns the result as an arbitrary-precision integer. |
| `$f64_parse` | Rounds an ASCII decimal token recognized by the spec to binary64, limiting its explicit exponent to ±19,999 as Zend does. |
| `$f64_format` | Produces Zend’s output bytes. Precision `14` gives ordinary float text. Precision `-1` gives the shortest round-tripping digits used in conversion diagnostics. |

## Functions supplied by P4-SpecTec

The spec also calls eight standard `builtin` functions. `$band`, `$bor`, `$bxor`, and `$bneg` compute bitwise AND, OR, XOR, and complement. `$shl` and `$shr` shift integers. `$min_nat` finds the shorter length when comparing two byte strings. `$find_map` looks up a registered byte-string name in the supplied constant table. [Bitwise rules](../spec/01-values/04-bitwise.watsup) · [Comparison rules](../spec/01-values/05-comparison.watsup) · [Constant lookup](../spec/01-values/16-builtin-constants.watsup)

## Checking the inventory

From the project root, list the declarations and their signatures directly from the elaborated spec.

```sh
../p4-spectec/bin/p4spectec elab spec | rg '^[[:space:]]*(extern|builtin)[[:space:]]'
```

To find individual call sites, search for the function name in `spec/`, for example with `rg -n -F '$f64_format(' spec`.
