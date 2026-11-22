# PHP Zend Opcode Specification

The PHP CLI extension prints compiled PHP as a `.zop` listing and loads listings for execution. The Rust library exposes `parse_zop` and `print_zop` to convert between listings and the Zend compilation-unit AST. [P4-SpecTec syntax](spec/syntax.watsup)

## Build and test

Use Rust 1.97.1, Python 3.10 or later, and a PHP build from the `php-8.6.0` tag with OPcache support. Place the P4-SpecTec checkout at `../p4-spectec` and put PHP's matching `phpize` and `php-config` on `PATH`. Run the commands below from the project root.

```sh
(cd ext && phpize && ./configure && make)
./scripts/test_extension.py
cargo test
```

Both test suites discover `.php`, `.phpt`, and `.zop` files under `tests/` and check them with OPcache off and with its default optimizer. They use `../php-src/sapi/cli/php` by default. Set `ZEND_SPEC_PHP` to select a matching executable elsewhere. Use `cargo test --lib` for the Rust unit tests alone.

## Print and run a listing

Use the matching `php` CLI. The first command compiles a file without executing it. The second runs the saved listing.

```sh
php -n -d extension="$PWD/ext/modules/zend_opcodes.so" -d zend_opcodes.output=example.zop -l example.php
php -n -d extension="$PWD/ext/modules/zend_opcodes.so" example.zop
```

Use listings from trusted sources with the same PHP build. [Hand-written example](tests/project/handwritten.zop)
