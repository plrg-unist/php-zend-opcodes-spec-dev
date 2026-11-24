#ifdef HAVE_CONFIG_H
#include "config.h"
#endif

#include "opcodes.h"
#include "SAPI.h"
#include "ext/standard/php_string.h"
#include "main/fopen_wrappers.h"
#include "main/php_ini.h"
#include "main/php_open_temporary_file.h"

static zend_op_array *(*previous_compile_file)(zend_file_handle *, int);

PHP_INI_BEGIN()
    PHP_INI_ENTRY("zend_opcodes.output", "", PHP_INI_SYSTEM, NULL)
    PHP_INI_ENTRY("zend_opcodes.load", "0", PHP_INI_SYSTEM, NULL)
PHP_INI_END()

static const char *const installation_names[] = {
    "DEFAULT_INCLUDE_PATH", "PEAR_INSTALL_DIR", "PHP_BINARY", "PHP_BINDIR", "PHP_BUILD_DATE",
    "PHP_CONFIG_FILE_PATH", "PHP_CONFIG_FILE_SCAN_DIR", "PHP_DATADIR", "PHP_EXTENSION_DIR",
    "PHP_LIBDIR", "PHP_LOCALSTATEDIR", "PHP_MANDIR", "PHP_PREFIX", "PHP_SBINDIR", "PHP_SYSCONFDIR"
};
#define INSTALLATION_COUNT (sizeof(installation_names) / sizeof(installation_names[0]))

/* Keep each persistent string alive while the request supplies its replacement. */
ZEND_TLS struct {
    zval *constant;
    zend_string *original;
} saved_installation[INSTALLATION_COUNT + 1]; /* PEAR_EXTENSION_DIR shares PHP_EXTENSION_DIR. */
ZEND_TLS size_t saved_installation_count;

static void replace_installation_string(const char *name, zend_string *value)
{
    zval *constant = zend_get_constant_str(name, strlen(name));
    if (!constant || Z_TYPE_P(constant) != IS_STRING) {
        zend_error_noreturn(E_COMPILE_ERROR, "Missing installation string constant %s", name);
    }
    saved_installation[saved_installation_count].constant = constant;
    saved_installation[saved_installation_count++].original = Z_STR_P(constant);
    ZVAL_STR(constant, zend_string_copy(value));
}

static bool decode_installation(char *input, size_t length, zend_string **values)
{
    if (!length || input[length - 1] != '\0') {
        return false;
    }
    size_t count = 0;
    for (char *name = input; name < input + length;) {
        char *separator = strchr(name, '=');
        if (!separator) {
            return false;
        }
        *separator = '\0';
        char *value = separator + 1;
        size_t i = 0;
        while (i < INSTALLATION_COUNT && strcmp(installation_names[i], name)) {
            i++;
        }
        if (i == INSTALLATION_COUNT || values[i]) {
            return false;
        }
        size_t value_length = strlen(value);
        values[i] = zend_string_init_interned(value, value_length, false);
        count++;
        name = value + value_length + 1;
    }
    return count == INSTALLATION_COUNT;
}

static void apply_installation(void)
{
    const zval *path = cfg_get_entry(ZEND_STRL("zend_opcodes.installation_file"));
    if (!path || saved_installation_count) {
        return;
    }
    if (Z_TYPE_P(path) != IS_STRING || zend_str_has_nul_byte(Z_STR_P(path)) ||
        php_check_open_basedir(Z_STRVAL_P(path))) {
        zend_error_noreturn(E_COMPILE_ERROR, "Invalid zend_opcodes.installation_file path");
    }
    FILE *fp = VCWD_FOPEN(Z_STRVAL_P(path), "rb");
    if (!fp) {
        zend_error_noreturn(E_COMPILE_ERROR, "Cannot open zend_opcodes.installation_file");
    }
    zend_file_handle file;
    zend_stream_init_fp(&file, fp, NULL);
    char *contents;
    size_t length;
    zend_string *values[INSTALLATION_COUNT] = {0};
    bool valid = zend_stream_fixup(&file, &contents, &length) == SUCCESS && !ferror(fp) &&
        decode_installation(contents, length, values);
    zend_destroy_file_handle(&file);
    for (size_t i = 0; i < INSTALLATION_COUNT; i++) {
        if (valid) {
            replace_installation_string(installation_names[i], values[i]);
            if (!strcmp(installation_names[i], "PHP_EXTENSION_DIR")) {
                replace_installation_string("PEAR_EXTENSION_DIR", values[i]);
            }
        }
        if (values[i]) {
            zend_string_release(values[i]);
        }
    }
    if (!valid) {
        zend_error_noreturn(E_COMPILE_ERROR, "Invalid zend_opcodes.installation_file contents");
    }
}

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
        apply_installation();
        char *contents;
        size_t length;
        if (zend_stream_fixup(file, &contents, &length) == FAILURE) {
            zend_error_noreturn(E_COMPILE_ERROR, "Cannot read opcode listing %s", ZSTR_VAL(file->filename));
        }
        HashTable *bindings = zend_new_array(0);
        zo_reader r = {.p = contents, .end = contents + length, .path = file->filename,
            .line = 1, .array_bindings = bindings};
        zend_try {
            main = zo_load_unit(&r);
        } zend_catch {
            zend_array_release(bindings);
            zend_bailout();
        } zend_end_try();
        zend_array_release(bindings);
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
    while (saved_installation_count) {
        size_t i = --saved_installation_count;
        zval *constant = saved_installation[i].constant;
        zend_string_release(Z_STR_P(constant));
        ZVAL_STR(constant, saved_installation[i].original);
    }
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
