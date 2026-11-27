# Execution inputs and outputs

A run takes four inputs.

- One compiled unit, supplied as a `.zop` listing.
- The path used to invoke the listing, visible to PHP as `$argv[0]`. Diagnostics use the source filename stored in the unit.
- The raw bytes on stdin, followed by EOF.
- The predefined runtime constants for this request. The repository JSON file supplies the 549 non-core bindings for the reference CLI. The semantics supplies the global names `TRUE`, `FALSE`, and `NULL`. [Reference table](../runtime-constants.json)

By default the executable forwards PHP's stdout, stderr, and exit status. With `--json` it writes one result to stdout and exits with status 0. The `stdout` and `stderr` fields hold arrays of byte values.

| Result `kind` | Meaning |
| --- | --- |
| `php` | The result gives `stdout`, `stderr`, and PHP's `exit_status`, whether the program exits normally or with a PHP fatal error. |
| `unsupported` | Execution needed an operation the spec does not handle. The result retains `stdout` and `stderr` emitted up to that point. |
| `malformed` | A diagnostic `message` explains why the executable rejected the listing before execution. |
| `tool_error` | A diagnostic `message` explains a failure in the executable, such as being unable to open the listing. |

## Runtime constants

The runner reads the checked-in reference table when `--constants FILE` is omitted. An explicit file supplies the entire non-core table. A missing key starts undefined, and a key mapped to JSON `null` starts defined with the PHP value `null`. A listing uses its own stored value for a literal folded during compilation. [Runtime and folded reads](../tests/project/runtime_constant_inputs.php)

The file is a JSON object keyed by constant name. Each repeatable `--constant NAME=JSON` adds or overrides a value after the file is loaded. For example, `--constant GLOB_MARK=2` supplies an integer and `--constant 'PHP_BINARY="2"'` supplies a byte string. Use a file for names containing an equals sign or NUL. Names use UTF-8. Lookup follows Zend's ASCII case folding for the namespace prefix. Any spelling of the three core global names is reserved.

| JSON value | PHP value |
| --- | --- |
| `null`, `true`, `false` | Null or Boolean |
| `9223372036854775807`, `1.0` | Signed 64-bit integer or binary64 float. A decimal point or exponent denotes a float. |
| `"text"`, `{"bytes":[0,255]}` | UTF-8 bytes or explicit octets |
| `{"float_bits":"8000000000000000"}` | Exact binary64 encoding as 16 hexadecimal digits, here negative zero |
| `{"stream":2}` | Stream resource with this positive PHP handle. A differential run requires an open stream at that handle in Zend. |

A file can attach a deprecation to a value, for example `{"value":2048,"deprecated":{"since":"8.4","message":"the error level was removed"}}`. Both text fields are optional. An entry such as `{"alias":"PHP_EXTENSION_DIR"}` shares a directly named value in that file. An override through either name updates the shared value and preserves its deprecation. Conflicting explicit overrides are input errors. The legacy installation flags accept raw bytes through the same override mechanism. [Reference examples](../runtime-constants.json)
