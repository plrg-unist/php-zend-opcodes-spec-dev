#include "opcodes.h"
#include "Zend/zend_attributes.h"
#include "ext/json/php_json_parser.h"
#include "main/fopen_wrappers.h"
#include "main/php_ini.h"

ZEND_TLS HashTable *native_constants;

/* Parse JSON objects as string-keyed maps. Constant names can start with NUL
 * or contain only digits. JSON arrays use the parser's ordinary packed lists. */
static int create_json_object(php_json_parser *parser, zval *object)
{
    array_init(object);
    zend_hash_real_init_mixed(Z_ARRVAL_P(object));
    return SUCCESS;
}

static int add_json_member(php_json_parser *parser, zval *object, zend_string *key, zval *value)
{
    /* PHP's JSON scanner promotes overflowing integers to doubles. A float
     * token has a decimal point or an exponent; an integer here is invalid. */
    const char *token = (const char *) parser->scanner.token;
    bool rounded_integer = Z_TYPE_P(value) == IS_DOUBLE &&
        strspn(token, "-0123456789") == (size_t) (parser->scanner.cursor - parser->scanner.token);
    bool added = !rounded_integer && zend_hash_add(Z_ARRVAL_P(object), key, value) != NULL;
    zend_string_release(key);
    if (!added) {
        zval_ptr_dtor(value);
        zval_ptr_dtor(object);
        return FAILURE;
    }
    return SUCCESS;
}

static HashTable *json_object(const zval *value)
{
    if (Z_TYPE_P(value) == IS_ARRAY && HT_IS_INITIALIZED(Z_ARRVAL_P(value)) &&
        !HT_IS_PACKED(Z_ARRVAL_P(value))) {
        return Z_ARRVAL_P(value);
    }
    return NULL;
}

static bool decode_bytes(zval *input, zval *value)
{
    if (Z_TYPE_P(input) != IS_ARRAY || json_object(input)) {
        return false;
    }
    size_t length = zend_hash_num_elements(Z_ARRVAL_P(input));
    zend_string *str = zend_string_alloc(length, false);
    size_t i = 0;
    zval *byte;
    ZEND_HASH_FOREACH_VAL(Z_ARRVAL_P(input), byte) {
        if (Z_TYPE_P(byte) != IS_LONG || Z_LVAL_P(byte) < 0 || Z_LVAL_P(byte) > 255) {
            zend_string_release(str);
            return false;
        }
        ZSTR_VAL(str)[i++] = (char) Z_LVAL_P(byte);
    } ZEND_HASH_FOREACH_END();
    ZSTR_VAL(str)[length] = '\0';
    ZVAL_STR(value, str);
    return true;
}

static bool decode_value(zval *input, zval *value, zend_long *unavailable_stream)
{
    HashTable *tag = json_object(input);
    if (!tag) {
        if (Z_TYPE_P(input) == IS_ARRAY || (Z_TYPE_P(input) == IS_DOUBLE && !zend_finite(Z_DVAL_P(input)))) {
            return false;
        }
        ZVAL_COPY(value, input);
        return true;
    }
    if (zend_hash_num_elements(tag) != 1) {
        return false;
    }
    zval *field = zend_hash_str_find(tag, ZEND_STRL("bytes"));
    if (field) {
        return decode_bytes(field, value);
    }
    field = zend_hash_str_find(tag, ZEND_STRL("float_bits"));
    if (field && Z_TYPE_P(field) == IS_STRING && Z_STRLEN_P(field) == 16 &&
        strspn(Z_STRVAL_P(field), "0123456789abcdefABCDEF") == 16) {
        uint64_t bits = strtoull(Z_STRVAL_P(field), NULL, 16);
        double number;
        memcpy(&number, &bits, sizeof(number));
        ZVAL_DOUBLE(value, number);
        return true;
    }
    field = zend_hash_str_find(tag, ZEND_STRL("stream"));
    if (field && Z_TYPE_P(field) == IS_LONG && Z_LVAL_P(field) > 0) {
        zval *stream = zend_hash_index_find(&EG(regular_list), Z_LVAL_P(field));
        if (stream && Z_TYPE_P(stream) == IS_RESOURCE &&
            (Z_RES_P(stream)->type == php_file_le_stream() || Z_RES_P(stream)->type == php_file_le_pstream())) {
            ZVAL_COPY(value, stream);
            return true;
        }
        *unavailable_stream = Z_LVAL_P(field);
    }
    return false;
}

static bool add_constant(HashTable *table, zend_string *name, zval *input, zend_long *unavailable_stream)
{
    HashTable *object = json_object(input);
    zval *metadata = object ? zend_hash_str_find(object, ZEND_STRL("deprecated")) : NULL;
    zval *since = NULL, *message = NULL;
    uint32_t argc = 0;
    if (metadata) {
        HashTable *deprecated = json_object(metadata);
        input = zend_hash_str_find(object, ZEND_STRL("value"));
        if (!deprecated || !input || zend_hash_num_elements(object) != 2) {
            return false;
        }
        since = zend_hash_str_find(deprecated, ZEND_STRL("since"));
        message = zend_hash_str_find(deprecated, ZEND_STRL("message"));
        argc = (since != NULL) + (message != NULL);
        if ((since && Z_TYPE_P(since) != IS_STRING) || (message && Z_TYPE_P(message) != IS_STRING) ||
            zend_hash_num_elements(deprecated) != argc) {
            return false;
        }
    }

    zval value;
    if (!decode_value(input, &value, unavailable_stream)) {
        return false;
    }
    /* zend_register_constant folds only the namespace prefix before the first
     * NUL. Building the initial table must not emit PHP declaration diagnostics. */
    zend_string *key = zend_string_copy(name);
    const char *slash = strrchr(ZSTR_VAL(name), '\\');
    if (slash) {
        key = zend_string_separate(key, false);
        zend_str_tolower(ZSTR_VAL(key), slash - ZSTR_VAL(name));
    }
    zend_constant *constant = ecalloc(1, sizeof(*constant));
    bool added = zend_hash_add_ptr(table, key, constant) != NULL;
    zend_string_release(key);
    if (!added) {
        efree(constant);
        zval_ptr_dtor(&value);
        return false;
    }
    constant->name = zend_string_copy(name);
    ZVAL_COPY_VALUE(&constant->value, &value);
    ZEND_CONSTANT_SET_FLAGS(constant, metadata ? CONST_DEPRECATED : 0, 0);
    if (metadata) {
        zend_attribute *attribute = zend_add_attribute(&constant->attributes,
            ZSTR_KNOWN(ZEND_STR_DEPRECATED_CAPITALIZED), argc, 0, 0, 0);
        if (since) {
            attribute->args[0].name = ZSTR_KNOWN(ZEND_STR_SINCE);
            ZVAL_COPY(&attribute->args[0].value, since);
        }
        if (message) {
            attribute->args[since != NULL].name = ZSTR_KNOWN(ZEND_STR_MESSAGE);
            ZVAL_COPY(&attribute->args[since != NULL].value, message);
        }
    }
    return true;
}

static HashTable *decode_constants(const char *contents, size_t length, zend_long *unavailable_stream)
{
    zval root;
    php_json_parser parser;
    php_json_parser_init(&parser, &root, contents, length, 0, PHP_JSON_PARSER_DEFAULT_DEPTH);
    parser.methods.object_create = create_json_object;
    parser.methods.object_update = add_json_member;
    if (php_json_parse(&parser)) {
        return NULL;
    }
    HashTable *entries = json_object(&root);
    HashTable *table = NULL;
    if (entries) {
        table = zend_new_array(zend_hash_num_elements(entries) + 3);
        table->pDestructor = EG(zend_constants)->pDestructor;
        static const char *const core_names[] = {"TRUE", "FALSE", "NULL"};
        for (size_t i = 0; i < sizeof(core_names) / sizeof(core_names[0]); i++) {
            zend_constant *core = zend_get_special_const(core_names[i], strlen(core_names[i]));
            add_constant(table, core->name, &core->value, unavailable_stream);
        }
        zend_string *name;
        zval *value;
        ZEND_HASH_FOREACH_STR_KEY_VAL(entries, name, value) {
            if (zend_get_special_const(ZSTR_VAL(name), ZSTR_LEN(name)) ||
                !add_constant(table, name, value, unavailable_stream)) {
                zend_array_destroy(table);
                table = NULL;
                break;
            }
        } ZEND_HASH_FOREACH_END();
    }
    zval_ptr_dtor(&root);
    return table;
}

void zo_apply_constants(zend_string *halt_key, zend_long halt_offset)
{
    const zval *path = cfg_get_entry(ZEND_STRL("zend_opcodes.constants_file"));
    if (!path || native_constants) {
        return;
    }
    if (Z_TYPE_P(path) != IS_STRING || zend_str_has_nul_byte(Z_STR_P(path)) ||
        php_check_open_basedir(Z_STRVAL_P(path))) {
        zend_error_noreturn(E_COMPILE_ERROR, "Invalid zend_opcodes.constants_file path");
    }
    FILE *fp = VCWD_FOPEN(Z_STRVAL_P(path), "rb");
    if (!fp) {
        zend_error_noreturn(E_COMPILE_ERROR, "Cannot open zend_opcodes.constants_file");
    }
    zend_file_handle file;
    zend_stream_init_fp(&file, fp, NULL);
    char *contents;
    size_t length;
    HashTable *table = NULL;
    zend_long unavailable_stream = 0;
    if (zend_stream_fixup(&file, &contents, &length) == SUCCESS && !ferror(fp)) {
        table = decode_constants(contents, length, &unavailable_stream);
    }
    zend_destroy_file_handle(&file);
    if (!table) {
        if (unavailable_stream) {
            zend_error_noreturn(E_COMPILE_ERROR,
                "No live PHP stream with resource ID " ZEND_LONG_FMT " in zend_opcodes.constants_file %s",
                unavailable_stream, Z_STRVAL_P(path));
        }
        zend_error_noreturn(E_COMPILE_ERROR, "Invalid zend_opcodes.constants_file contents in %s", Z_STRVAL_P(path));
    }
    if (halt_key) {
        /* The finalized unit owns its file-specific halt offset. */
        zval offset;
        ZVAL_LONG(&offset, halt_offset);
        zend_hash_del(table, halt_key);
        add_constant(table, halt_key, &offset, &unavailable_stream);
    }
    native_constants = EG(zend_constants);
    EG(zend_constants) = table;
}

void zo_restore_constants(void)
{
    if (native_constants) {
        HashTable *table = EG(zend_constants);
        EG(zend_constants) = native_constants;
        native_constants = NULL;
        zend_array_destroy(table);
    }
}
