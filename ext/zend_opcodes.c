#ifdef HAVE_CONFIG_H
#include "config.h"
#endif

#include "opcodes.h"
#include "SAPI.h"
#include "ext/standard/php_string.h"
#include "main/fopen_wrappers.h"
#include "main/php_open_temporary_file.h"

static zend_op_array *(*previous_compile_file)(zend_file_handle *, int);

PHP_INI_BEGIN()
    PHP_INI_ENTRY("zend_opcodes.output", "", PHP_INI_SYSTEM, NULL)
    PHP_INI_ENTRY("zend_opcodes.load", "0", PHP_INI_SYSTEM, NULL)
PHP_INI_END()

static void write_listing(const char *destination, zend_string *input, zend_string *listing)
{
    char path[MAXPATHLEN], source[MAXPATHLEN];
    if (!expand_filepath(destination, path) || php_check_open_basedir(path)) {
        zend_error_noreturn(E_COMPILE_ERROR, "Cannot write opcode listing to %s", destination);
    }
    if (expand_filepath(ZSTR_VAL(input), source) && strcmp(path, source) == 0) {
        zend_error_noreturn(E_COMPILE_ERROR, "Opcode listing path is also the input file: %s", path);
    }
    char directory[MAXPATHLEN];
    memcpy(directory, path, strlen(path) + 1);
    php_dirname(directory, strlen(directory));
    zend_string *temporary = NULL;
    FILE *fp = php_open_temporary_file(directory, ".zend-opcodes-", &temporary);
    if (!fp) {
        zend_error_noreturn(E_COMPILE_ERROR, "Cannot create opcode listing in %s", directory);
    }
    bool ok = fwrite(ZSTR_VAL(listing), 1, ZSTR_LEN(listing), fp) == ZSTR_LEN(listing);
    if (fclose(fp) != 0 || !ok || VCWD_RENAME(ZSTR_VAL(temporary), path) != 0) {
        VCWD_UNLINK(ZSTR_VAL(temporary));
        zend_string_release(temporary);
        zend_error_noreturn(E_COMPILE_ERROR, "Cannot write opcode listing to %s", path);
    }
    zend_string_release(temporary);
}

static zend_op_array *compile_unit(zend_file_handle *file, int type)
{
    if (!file->primary_script) {
        return previous_compile_file(file, type);
    }
    const char *output = zend_ini_string_literal("zend_opcodes.output");
    bool load = zend_ini_bool_literal("zend_opcodes.load") || zend_string_ends_with_literal(file->filename, ".zop");
    if (!load && (!output || !*output)) {
        return previous_compile_file(file, type);
    }
    uint32_t functions_start = CG(function_table)->nNumUsed;
    uint32_t classes_start = CG(class_table)->nNumUsed;
    uint8_t autoglobals_before = zo_armed_autoglobals();
    zend_op_array *main;
    if (load) {
        char *contents;
        size_t length;
        if (zend_stream_fixup(file, &contents, &length) == FAILURE) {
            zend_error_noreturn(E_COMPILE_ERROR, "Cannot read opcode listing %s", ZSTR_VAL(file->filename));
        }
        zo_reader r = {.p = contents, .end = contents + length, .path = file->filename, .line = 1};
        main = zo_load_unit(&r);
    } else {
        main = previous_compile_file(file, type);
    }
    if (main && output && *output) {
        smart_str listing = {0};
        zo_print_unit(&listing, main, functions_start, classes_start, autoglobals_before);
        write_listing(output, file->filename, listing.s);
        smart_str_free(&listing);
    }
    return main;
}

PHP_MINIT_FUNCTION(zend_opcodes)
{
    REGISTER_INI_ENTRIES();
    return SUCCESS;
}

PHP_MSHUTDOWN_FUNCTION(zend_opcodes)
{
    UNREGISTER_INI_ENTRIES();
    return SUCCESS;
}

PHP_RINIT_FUNCTION(zend_opcodes)
{
#if defined(ZTS) && defined(COMPILE_DL_ZEND_OPCODES)
    ZEND_TSRMLS_CACHE_UPDATE();
#endif
    if (strcmp(sapi_module.name, "cli") == 0) {
        previous_compile_file = zend_compile_file;
        zend_compile_file = compile_unit;
    }
    return SUCCESS;
}

PHP_RSHUTDOWN_FUNCTION(zend_opcodes)
{
    if (zend_compile_file == compile_unit) {
        zend_compile_file = previous_compile_file;
    }
    return SUCCESS;
}

zend_module_entry zend_opcodes_module_entry = {
    STANDARD_MODULE_HEADER,
    "zend_opcodes", NULL,
    PHP_MINIT(zend_opcodes), PHP_MSHUTDOWN(zend_opcodes),
    PHP_RINIT(zend_opcodes), PHP_RSHUTDOWN(zend_opcodes),
    NULL, NO_VERSION_YET, STANDARD_MODULE_PROPERTIES
};

#ifdef COMPILE_DL_ZEND_OPCODES
#ifdef ZTS
ZEND_TSRMLS_CACHE_DEFINE()
#endif
ZEND_GET_MODULE(zend_opcodes)
#endif
