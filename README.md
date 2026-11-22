# PHP Zend Opcode Specification

This project aims to specify PHP's Zend opcodes using P4-SpecTec. Its PHP CLI extension prints a compiled PHP file as text and loads a text listing back into Zend for execution.

## Build and test

Use a PHP build from the `php-8.6.0` tag with OPcache support. Put its matching `phpize` and `php-config` on `PATH`. From the project root, build the module and run the tests with Python 3.10 or later.

```sh
(cd ext && phpize && ./configure && make)
./scripts/test_extension.py
```

The runner uses `../php-src/sapi/cli/php` by default. Set `ZEND_SPEC_PHP` to use a matching executable elsewhere. It finds `.php`, `.phpt`, and `.zop` files anywhere under `tests/` and checks them with OPcache off and with its default optimizer.

## Print and run a listing

Use the matching `php` CLI. The first command compiles a file without executing it. The second runs the saved listing.

```sh
php -n -d extension="$PWD/ext/modules/zend_opcodes.so" -d zend_opcodes.output=example.zop -l example.php
php -n -d extension="$PWD/ext/modules/zend_opcodes.so" example.zop
```

Use listings from trusted sources with the same PHP build. [Hand-written example](tests/project/handwritten.zop)
