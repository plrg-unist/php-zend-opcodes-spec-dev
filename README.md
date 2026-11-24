# PHP Zend Opcode Specification

The PHP extension prints compiled PHP as a `.zop` listing and loads listings into Zend. The Rust library converts between listings and the compilation-unit AST. The `zend-opcodes` executable runs the supported main-body instructions using P4-SpecTec. [Intentional discrepancies and foreign calls](design-docs/README.md) · [Syntax](spec/00-syntax.watsup) · [Execution rules](spec/02-vm/00-core.watsup)

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

`target/debug/zend-opcodes --php-binary /example/php tests/project/installation_php_binary.zop` prints `/example/php`. Each installation constant has a lowercase flag with underscores replaced by hyphens. Runtime lookups read the supplied value. Literals in the listing keep their compiled bytes. [Default values and aliases](installation-defaults.json)

Use listings from trusted sources with the matching PHP build when loading them into Zend.
