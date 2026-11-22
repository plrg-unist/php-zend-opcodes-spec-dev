# PHP Zend Opcode Specification

This project aims to specify PHP's Zend opcodes using P4-SpecTec. The first milestone is a PHP extension that prints opcode programs and loads them back into the engine.

## Planned build and test

Once the extension and test runner are implemented, use PHP built from the `php-8.6.0` tag with OPcache support. Put the matching `phpize` and `php-config` on `PATH`. Run these commands from the project root.

```sh
(cd ext && phpize && ./configure && make)
./scripts/test_extension.py
```

The runner will use `../php-src/sapi/cli/php` by default. Set `ZEND_SPEC_PHP` to use a matching PHP executable elsewhere. One run will check opcode round trips and execution results with OPcache off and with its default optimizer.
