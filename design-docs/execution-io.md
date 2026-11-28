# Execution inputs and outputs

A run takes five inputs.

- One compiled unit, supplied as a `.zop` listing.
- The path used to invoke the listing, visible to PHP as `$argv[0]`. Diagnostics use the source filename stored in the unit.
- The raw bytes on stdin, followed by EOF.
- The predefined runtime constants for this request. The repository JSON file supplies the 549 non-core bindings for the reference CLI. The semantics supplies the global names `TRUE`, `FALSE`, and `NULL`.
- The size in bytes of a complete Zend instruction record (`sizeof(zend_op)`) on the target build. The reference build uses 32. [Reference inputs](../runtime-inputs.json)

By default the executable forwards PHP's stdout, stderr, and exit status. With `--json` it writes one result to stdout and exits with status 0. The `stdout` and `stderr` fields hold arrays of byte values.

| Result `kind` | Meaning |
| --- | --- |
| `php` | The result gives `stdout`, `stderr`, and PHP's `exit_status`, whether the program exits normally or with a PHP fatal error. |
| `unsupported` | Execution needed an operation the spec does not handle. The result retains `stdout` and `stderr` emitted up to that point. |
| `malformed` | A diagnostic `message` explains why the executable rejected the listing before execution. |
| `tool_error` | A diagnostic `message` explains a failure in the executable, such as being unable to open the listing. |

## Runtime inputs

The runner reads `runtime-inputs.json` from the repository by default. An explicit `--inputs FILE` supplies a complete replacement. The file must be a JSON object with exactly the two named fields shown in `{"opline_size_bytes":32,"constants":{"GLOB_MARK":8}}`. Each repeatable `--input KEY=JSON` applies after the file is loaded. Conflicting explicit overrides for the same input are errors. [Reference inputs](../runtime-inputs.json)

`opline_size_bytes` must be a JSON integer from 1 through 2147483647. Branches use instruction numbers. When a program reads a switch or match jump table as an ordinary array, each entry gives the exact byte distance from the dispatch instruction to the target. With `--input opline_size_bytes=56`, an entry targeting the next instruction reads as `56`. Before execution, the runner checks that every table entry's distance fits in a signed 32-bit integer (−2147483648 through 2147483647). A listing with an out-of-range entry is `malformed` for the selected size, even if the dispatch instruction never runs. [Jump-table fixture](../tests/project/array_dispatch_offsets.zop) · [Backward limit](../tests/project/array_dispatch_backward_offset.zop)

## Runtime constants

The `constants` object supplies the entire non-core table. A missing key starts undefined, and a key mapped to JSON `null` starts defined with the PHP value `null`. A listing uses its own stored value for a literal folded during compilation. [Runtime and folded reads](../tests/project/runtime_constant_inputs.php)

Use `--input constants.NAME=JSON` to add or override a PHP constant. For example, `--input constants.GLOB_MARK=2` supplies an integer and `--input 'constants.PHP_BINARY="2"'` supplies a byte string. The entire name after `constants.` can include periods. Use a file for names containing an equals sign or NUL. Names use UTF-8. Lookup follows Zend's ASCII case folding for the namespace prefix. Any spelling of the three core global names is reserved.

| JSON value | PHP value |
| --- | --- |
| `null`, `true`, `false` | Null or Boolean |
| `9223372036854775807`, `1.0` | Signed 64-bit integer or binary64 float. A decimal point or exponent denotes a float. |
| `"text"`, `{"bytes":[0,255]}` | UTF-8 bytes or explicit octets |
| `{"float_bits":"8000000000000000"}` | Exact binary64 encoding as 16 hexadecimal digits, here negative zero |
| `{"stream":42}` | Stream resource with this positive PHP handle. [Stream handles as input](#stream-handles-as-input) |

A file can attach a deprecation to a value, for example `{"value":2048,"deprecated":{"since":"8.4","message":"the error level was removed"}}`. Both text fields are optional. An entry such as `{"alias":"PHP_EXTENSION_DIR"}` shares a directly named value in that file. An override through either name updates the shared value and preserves its deprecation. [Reference examples](../runtime-inputs.json)

### Stream handles as input

The standalone spec accepts a positive handle as a resource identity. Only stdin, stdout and stderr need I/O endpoints within scope. Our native test loader resolves the JSON handle against Zend’s live stream resources because a PHP stream value refers to an underlying resource. [Execution scope](scope.md) · [Native loader](../ext/constants.c#L93)

For example, supply `--input 'constants.PHP_BINARY={"stream":42}'` to the listing for `echo PHP_BINARY;`. The spec prints `Resource id #42` without a newline and exits 0. When handle 42 is unused, the native loader reports `No live PHP stream with resource ID 42` on stdout and exits 255 before the listing runs. Both runs have empty stderr under the reference settings. [Listing](../tests/project/installation_php_binary.zop) · [CLI regression](../tests/cli.rs)
