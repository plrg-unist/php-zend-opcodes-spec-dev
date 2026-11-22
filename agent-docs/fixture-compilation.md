# PHP build and optimizer settings for fixtures

## External PHP build

Build the PHP engine from the external php-src checkout. The sibling checkout supplies upstream `php/php-src` at tag `php-8.6.0`, commit `8aaada3f9605d81a38b8240f93c7389dfbc77ef9`.

The fixture driver defaults to `../php-src/sapi/cli/php` relative to the project root. `ZEND_SPEC_PHP` selects another executable, e.g. `/opt/php-8.6.0/bin/php`. The default is the product of an in-tree build. Match the source commit, architecture, configuration and enabled extensions when sharing fixtures. Multiple source revisions can report the same PHP release.

## Extension layout and build

The extension lives in `ext/`. Its build definition is `config.m4`. The module and CLI compiler hook live in `zend_opcodes.c`. Each text representation keeps its reader and writer together, including compilation units and declarations in `unit.c`, function bodies in `body.c`, and oplines in `instructions.c`. [Module](../ext/zend_opcodes.c) · [Build definition and source list](../ext/config.m4)

Select `phpize` and `php-config` from the matching PHP development installation. A standard installation puts both tools in its `bin/` directory. For the source-tree CLI, use the tools under the matching installation prefix. The following commands start at the project root with those tools on `PATH`.

```sh
cd ext
phpize
./configure
make
```

`phpize` prepares `configure`, which reads the header paths from `php-config` and generates `ext/Makefile`. To select the helper explicitly, pass `--with-php-config=/path/to/pinned/php-config` to `./configure`. Subsequent C edits need only `make -C ext` from the project root. Keep generated build files out of Git. [Helper installation](../../php-src/scripts/Makefile.frag#L40) · [Header discovery](../../php-src/scripts/phpize.m4#L53) · [Makefile and module directory](../../php-src/build/php.m4#L140) · [PHP build guide](https://www.php.net/manual/en/install.pecl.phpize.php)

The runner loads `ext/modules/zend_opcodes.so` by absolute path with `-d extension=...`. PHP checks the module API and build ID. Use a consistent source path for fixtures that observe `__FILE__` or generated names. [Module loader](../../php-src/ext/standard/dl.c#L185)

## Finalized opcode variants

For an export-only run, use a named file and a fresh `php -n -d extension=/absolute/path/to/zend_opcodes.so -d zend_opcodes.output=out.zop -l fixture.php` process per profile. For OPcache profiles set `opcache.enable=1`, `opcache.enable_cli=1`, `opcache.file_update_protection=0` and keep JIT disabled. This permits newly written fixtures to reach the optimizer. The extension wraps the installed file compiler at request startup and exports after it returns. [File eligibility](../../php-src/ext/opcache/ZendAccelerator.c#L1804) · [Optimizing a script](../../php-src/ext/opcache/ZendAccelerator.c#L1616) · [Returning the loaded script](../../php-src/ext/opcache/ZendAccelerator.c#L2514)

The standard pair is the ordinary compiler (OPcache off) and OPcache with its pinned default optimizer. A local-only profile can be used for manual investigation. The probe observed the following complete instruction sequences for `answer()`.

```php
function answer() {
    $n = 40;
    if (false) { return 0; }
    return $n + 2;
}
```

| Profile | Setting | `answer()` body |
| --- | --- | --- |
| Ordinary compiler | `opcache.enable=0` | `ASSIGN, JMPZ, RETURN, ADD, RETURN, RETURN` |
| OPcache, local optimization only | `opcache.optimization_level=0x1` | `ASSIGN, JMP, RETURN, ADD, RETURN, RETURN` |
| OPcache, pinned default | `opcache.optimization_level=0x7FFEBFFF` | `RETURN int(42)` |

The same native extension observed all three finalized bodies. Two fresh processes per profile produced identical probe output. Executing the same file with `echo answer();` in a separate CLI process printed `42` under each profile.

For a controlled optimizer comparison, keep OPcache enabled and use mask `0` on one side. OPcache sets compiler flags before optimization. With a child class declared before its parent, the plain compiler emitted `DECLARE_CLASS` and OPcache with mask `0` emitted `DECLARE_CLASS_DELAYED`. [Compiler flags](../../php-src/ext/opcache/ZendAccelerator.c#L1852) · [Declaration emission](../../php-src/Zend/zend_compile.c#L10160)

`opcache.optimization_level` selects transformations such as jump simplification, constant propagation and dead-code elimination. `opcache.opt_debug_level` prints diagnostic views. On a fresh run, `0x20000` can confirm a file reached the optimizer even if its final instructions match another profile. Internally the optimizer temporarily reverses parts of `pass_two` and restores the VM encoding at finalization. To generate a runnable variant, select a mask and let compilation finish. Record the profile with its source fixture. Compare normalized program contents, excluding profile labels, when identifying duplicates. [Pinned pass definitions](../../php-src/Zend/Optimizer/zend_optimizer.h#L25) · [Finalization](../../php-src/Zend/Optimizer/zend_optimizer.c#L1460) · [OPcache settings](https://www.php.net/manual/en/opcache.configuration.php#ini.opcache.optimization-level)
