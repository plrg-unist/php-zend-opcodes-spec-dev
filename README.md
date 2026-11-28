# PHP Zend Opcode Specification

The PHP extension prints compiled PHP as a `.zop` listing and loads listings into Zend. The Rust library converts between listings and the compilation-unit AST. The `zend-opcodes` executable runs supported instructions in main and in directly called functions using P4-SpecTec. [Specification design](design-docs/README.md) · [Syntax](spec/00-syntax.watsup) · [Execution rules](spec/02-vm/00-core.watsup)

## Build and test

Use Rust 1.97.1, Python 3.10 or later, and a PHP build from the `php-8.6.0` tag with OPcache support. Place the P4-SpecTec checkout at `../p4-spectec` and put PHP's matching `phpize` and `php-config` on `PATH`. Run the commands below from the project root.

```sh
(cd ext && phpize && ./configure && make)
./scripts/test_extension.py
cargo test
./scripts/test_semantics.py
```

The extension and Cargo suites discover `.php`, `.phpt`, and `.zop` fixtures under `tests/`. All three suites use `../php-src/sapi/cli/php` by default. Set `ZEND_SPEC_PHP` to select a matching PHP elsewhere.

The semantics suite compares the spec with Zend in parallel under both compiler profiles. With no paths it runs all registered fixtures and requires 100% spec instruction coverage. For a focused run, use a command such as `./scripts/test_semantics.py -j 4 tests/project/standalone_branches.zop`. [Registered fixtures](tests/semantics.json)

## Print and run a listing

Use the matching `php` CLI. The first command compiles a file without executing it. The second runs the saved listing.

```sh
php -n -d extension="$PWD/ext/modules/zend_opcodes.so" -d zend_opcodes.output=example.zop -l example.php
php -n -d extension="$PWD/ext/modules/zend_opcodes.so" example.zop
```

Build the spec executable with `cargo build`. Running `target/debug/zend-opcodes tests/project/handwritten.zop </dev/null` prints `running=41`. The executable reads stdin through EOF. Add `--json` before the path for a structured outcome. [Hand-written example](tests/project/handwritten.zop)

The runner reads the target instruction size and predefined PHP constants from the repository's runtime inputs. Use `--inputs FILE` to select a complete JSON file and repeat `--input KEY=JSON` to override individual values. For example, `--input 'constants.PHP_BINARY="/example/php"'` supplies a constant, and `--input opline_size_bytes=56` selects 56-byte Zend instructions. [Reference inputs](runtime-inputs.json) · [Input format](design-docs/execution-io.md#runtime-inputs)

Use listings from trusted sources with the matching PHP build when loading them into Zend.
