# Zend opcode AST and Rust library

The Rust library converts a `.zop` listing to a P4-SpecTec compilation-unit value and back. The spec executable takes the same value as input. [AST definition](../spec/syntax.watsup) · [Concrete syntax](opcode-syntax.md) · [Execution contract](differential-testing.md)

## Specification and crate

- The top-level `unit` contains the filename, optional compiler halt offset, ordered lazy superglobals, main body, named functions and classes. Instructions use typed opcode and operand variants. Preserve order in bodies, literal pools and declarations. PHP strings are lists of octets. `FLOAT` holds the binary64 bits, including signed zero and NaN payloads. [Unit contents](opcode-input.md)
- The root Cargo package provides the library and `zend-opcodes` binary. It pins Rust `1.97.1` and depends on the sibling `../p4-spectec/p4spectec/core`. The binary loads the production `spec/` directory from the checkout used to build it. Changes to `.watsup` rules apply on the next invocation. [Manifest](../Cargo.toml) · [Toolchain](../rust-toolchain.toml) · [Execution host](../src/main.rs)
- `parse_zop(&mut Arena, &str)` returns a P4-SpecTec `Value` in the caller's arena. Pass that arena and value to `print_zop(&Arena, &Value)` to obtain a `String`. Both functions return a `Result`. Callers handle file I/O. [Library exports](../src/lib.rs) · [Parser](../src/parse.rs) · [Printer](../src/print.rs)

## Tests

- `cargo test --lib` compares small hand-written listings with independently authored P4-SpecTec terms in both conversion directions. Elaborate the terms together with the syntax. [Rust unit tests](../src/tests.rs) · [Expected terms](../src/tests.watsup)
- `cargo test --test roundtrip` discovers every `.php`, `.phpt` and `.zop` fixture under both compiler profiles. For each valid generated or hand-written listing, parse, print and reparse in the same arena. Check that both values inhabit the spec's `unit` type and compare them with `syntax_eq` on their value views. [Cargo integration test](../tests/roundtrip.rs) · [P4-SpecTec equality](../../p4-spectec/p4spectec/core/src/lang/data/value/flat/cmp.rs#L80)
- Execute the original and Rust-printed listings in fresh Zend processes at the same temporary script path. Compare stdout and stderr bytes, exit status and Zend's canonical reprints. A syntactically valid fixture with an out-of-range jump can produce an AST and still require loader rejection. [Fixture expectations and profiles](extension-testing.md)

The extension test imports `scripts/zend_fixtures.py` for discovery, PHPT handling, profiles and PHP invocation. The Cargo integration test resolves the executable helper from `CARGO_MANIFEST_DIR` and invokes it directly to prepare listings and compare Zend runs. Generated files live in automatically cleaned temporary directories. After building the extension, `cargo test` runs both test groups. [Shared fixture helper](../scripts/zend_fixtures.py)
