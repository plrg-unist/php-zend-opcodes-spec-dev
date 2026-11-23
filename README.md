# PHP Zend Opcode Specification

The PHP extension prints compiled PHP as a `.zop` listing and loads listings into Zend. The Rust library converts between listings and the compilation-unit AST. The `zend-opcodes` executable runs the supported main-body instructions using P4-SpecTec. [Design choices](design-docs/README.md) · [Syntax](spec/syntax.watsup) · [Execution rules](spec/vm.watsup)

## Build and test

Use Rust 1.97.1, Python 3.10 or later, and a PHP build from the `php-8.6.0` tag with OPcache support. Place the P4-SpecTec checkout at `../p4-spectec` and put PHP's matching `phpize` and `php-config` on `PATH`. Run the commands below from the project root.

```sh
(cd ext && phpize && ./configure && make)
./scripts/test_extension.py
cargo test
./scripts/test_semantics.py
```

The extension and Cargo suites discover `.php`, `.phpt`, and `.zop` fixtures under `tests/`. The semantics suite builds the executable, compares its results with Zend under both compiler profiles, and checks spec instruction coverage. It runs the [semantics cases](tests/semantics.json). All three suites use `../php-src/sapi/cli/php` by default. Set `ZEND_SPEC_PHP` to select a matching PHP elsewhere.

## Print and run a listing

Use the matching `php` CLI. The first command compiles a file without executing it. The second runs the saved listing.

```sh
php -n -d extension="$PWD/ext/modules/zend_opcodes.so" -d zend_opcodes.output=example.zop -l example.php
php -n -d extension="$PWD/ext/modules/zend_opcodes.so" example.zop
```

Build the spec executable with `cargo build`. Running `target/debug/zend-opcodes tests/project/handwritten.zop </dev/null` prints `running=41`. The executable reads stdin through EOF. Add `--json` before the path for a structured outcome. [Hand-written example](tests/project/handwritten.zop)

Use listings from trusted sources with the matching PHP build when loading them into Zend.
