# Zend opcode AST and Rust library

The next milestone covers the P4-SpecTec syntax for one Zend compilation unit and conversion between that AST and a `.zop` listing. Opcode execution semantics come later.

## Specification and crate

- Start with one `spec/syntax.watsup`. Model the full unit the extension can print, including typed opcodes and operands, ordered literal pools, nested bodies, declarations and metadata. Preserve PHP strings as bytes and binary64 values exactly, including signed zero and NaN payloads. [Unit contents](opcode-input.md) · [Concrete syntax](opcode-syntax.md)
- Use one unpublished, library-only Cargo package at the project root with `Cargo.toml`, `Cargo.lock` and `rust-toolchain.toml` pinning Rust `1.97.1`. Omit the optional package version. Use the sibling `../p4-spectec/p4spectec/core` path dependency. [Local dependency example](../../while-example/Cargo.toml)
- Expose `parse_zop(&mut Arena, &str)` and `print_zop(&Arena, &Value)` from `src/lib.rs`, with implementations in `src/parse.rs` and `src/print.rs`. Return the P4-SpecTec `Value` or printed `String` via `Result`. Build values directly in the caller's arena. File I/O belongs to the caller. [Example of constructing terms](../../while-example/src/grammar.lalrpop)

## Tests

- **Unit tests** (`src/tests.rs`, `cargo test --lib`). Hand-write a few small `.zop` programs and independent expected P4-SpecTec terms. Elaborate the expected terms against the syntax and compare them with the parsed ASTs. For example, `ECHO lit[0]` retains a reference to slot zero and the string bytes live in the literal pool.
- **Corpus integration test** (`tests/roundtrip.rs`, `cargo test --test roundtrip`). Use every discovered `.php`, `.phpt` and `.zop` fixture under both compiler profiles. Generate listings from PHP sources and use hand-written listings directly. For each valid listing, parse, print and reparse in the same arena, then compare the ASTs with `syntax_eq` on their value views. [P4-SpecTec equality](../../p4-spectec/p4spectec/core/src/lang/data/value/flat/cmp.rs#L80)
- Run the original and Rust-printed listings through Zend in fresh processes at the same temporary script path. Compare stdout and stderr byte-for-byte, exit status and Zend's canonical reprints. A syntactically valid fixture with an out-of-range jump can produce an AST and still require loader rejection. [Fixture expectations and profiles](extension-testing.md)

Keep `scripts/test_extension.py` as the extension test entry point. Extract discovery, PHPT section handling, profile settings and PHP invocation into `scripts/zend_fixtures.py`. The Python test imports it. The Cargo integration test resolves it from `CARGO_MANIFEST_DIR` and executes it directly to prepare listings and compare Zend runs. Set its executable bit and use `#!/usr/bin/env python3`. Keep generated files in automatically cleaned temporary directories. After building the extension, `cargo test` runs both test groups.
