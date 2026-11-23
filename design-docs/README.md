# What the opcode spec preserves

The pinned Zend CLI is the reference for how a compiled PHP program behaves. For the same opcodes and input, the default is exact agreement on stdout bytes, stderr bytes, and the process exit status. The model can omit an engine detail when no PHP program in scope can make that detail observable, even through code that runs later.

For example, a main file containing `echo "A"; return 7; echo "B";` prints `A` and exits with status `0`. The CLI discards the returned `7`, so the spec can leave that value out of its result. It must still end execution at the `return`, because reaching the second `echo` would change the output. [Execution rules](../spec/vm.watsup)

For string interpolation, Zend can spread the work across several *rope* instructions. The spec records each piece as bytes under its rope and position, leaving buffer allocation and reuse to the engine. It reads each value when its instruction runs, so an undefined variable produces a warning at the same point as in Zend. The final instruction joins the pieces in position order. [String rules](../spec/vm_strings.watsup) · [Interleaved ropes example](../tests/project/rope_indices.zop)

The spec stores a PHP float as its 64 bits. A small Rust helper performs binary64 arithmetic and decimal conversion. The PHP rules decide when to convert a value, which precision to request, and whether to warn or throw. This distinction is visible even for one float. Ordinary output uses the configured precision, while a precision-loss warning can print extra digits to identify the exact float. [Float operations](../spec/value_binary64.watsup) · [Formatting example](../tests/project/numeric_float_text.php)

The point at which integer arithmetic overflows can affect a later comparison. For example, `3 ** 61` and `3.0 ** 61` produce adjacent floating-point values in Zend, and PHP can distinguish them with `===`. The spec therefore keeps integer exponentiation in integers through each multiplication that fits and promotes where Zend does. [Power rules](../spec/values_numeric_arithmetic.watsup) · [Executable example](../tests/project/numeric_power_edges.php)
