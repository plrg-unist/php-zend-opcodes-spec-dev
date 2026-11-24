# Execution inputs and outputs

A run takes four inputs.

- One compiled unit, supplied as a `.zop` listing.
- The path used to invoke the listing, visible to PHP as `$argv[0]`. Diagnostics use the source filename stored in the unit.
- The raw bytes on stdin, followed by EOF.
- Byte-string values for 16 installation-specific PHP constants. The executable supplies defaults and accepts overrides through CLI flags. [Defaults and aliases](../installation-defaults.json)

By default the executable forwards PHP's stdout, stderr, and exit status. With `--json` it writes one result to stdout and exits with status 0. The `stdout` and `stderr` fields hold arrays of byte values.

| Result `kind` | Meaning |
| --- | --- |
| `php` | The result gives `stdout`, `stderr`, and PHP's `exit_status`, whether the program exits normally or with a PHP fatal error. |
| `unsupported` | Execution needed an operation the spec does not handle. The result retains `stdout` and `stderr` emitted up to that point. |
| `malformed` | A diagnostic `message` explains why the executable rejected the listing before execution. |
| `tool_error` | A diagnostic `message` explains a failure in the executable, such as being unable to open the listing. |
