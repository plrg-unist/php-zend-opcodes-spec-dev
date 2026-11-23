#include "opcodes.h"
#include "Zend/zend_strtod.h"

void zo_fail(zo_reader *r, const char *format, ...)
{
    va_list args;
    va_start(args, format);
    zend_string *message = zend_vstrpprintf(0, format, args);
    va_end(args);
    zend_error_noreturn(E_COMPILE_ERROR, "Opcode input %s:%u (%s): %s",
        ZSTR_VAL(r->path), r->line, r->body ? ZSTR_VAL(r->body) : "unit", ZSTR_VAL(message));
}

void zo_unsupported(const char *format, ...)
{
    va_list args;
    va_start(args, format);
    zend_string *message = zend_vstrpprintf(0, format, args);
    va_end(args);
    zend_error_noreturn(E_COMPILE_ERROR, "Cannot print opcode unit: %s", ZSTR_VAL(message));
}

static void skip_space(zo_reader *r)
{
    while (r->p != r->end && (*r->p == ' ' || *r->p == '\t')) {
        ++r->p;
    }
}

bool zo_take(zo_reader *r, const char *token)
{
    size_t size = strlen(token);
    skip_space(r);
    if ((size_t)(r->end - r->p) < size || memcmp(r->p, token, size) != 0) {
        return false;
    }
    r->p += size;
    return true;
}

void zo_expect(zo_reader *r, const char *token)
{
    if (!zo_take(r, token)) {
        zo_fail(r, "expected '%s'", token);
    }
}

void zo_eol(zo_reader *r)
{
    skip_space(r);
    if (r->p == r->end) {
        return;
    }
    if (*r->p == '\r') {
        ++r->p;
    }
    if (r->p == r->end || *r->p != '\n') {
        zo_fail(r, "expected end of line");
    }
    ++r->p;
    ++r->line;
}

static bool decimal_digit(char c)
{
    return c >= '0' && c <= '9';
}

static zend_ulong positive_magnitude(zo_reader *r, zend_ulong limit)
{
    zend_ulong value = 0;
    if (r->p == r->end || !decimal_digit(*r->p)) {
        zo_fail(r, "expected a decimal integer");
    }
    do {
        unsigned digit = (unsigned)(*r->p++ - '0');
        if (value > (limit - digit) / 10) {
            zo_fail(r, "integer exceeds its field's range");
        }
        value = value * 10 + digit;
    } while (r->p < r->end && decimal_digit(*r->p));
    return value;
}

uint32_t zo_u32(zo_reader *r)
{
    skip_space(r);
    return (uint32_t)positive_magnitude(r, UINT32_MAX);
}

zend_long zo_long(zo_reader *r)
{
    bool negative = zo_take(r, "-");
    zend_ulong magnitude = positive_magnitude(r, (zend_ulong)ZEND_LONG_MAX + negative);
    if (!negative) {
        return (zend_long)magnitude;
    }
    return magnitude > ZEND_LONG_MAX ? ZEND_LONG_MIN : -(zend_long)magnitude;
}

static const char hex_digits[] = "0123456789abcdef";

static unsigned read_hex(zo_reader *r)
{
    if (r->p < r->end) {
        char c = *r->p++;
        if (decimal_digit(c)) return (unsigned)(c - '0');
        if (c >= 'a' && c <= 'f') return (unsigned)(c - 'a') + 10;
        if (c >= 'A' && c <= 'F') return (unsigned)(c - 'A') + 10;
    }
    zo_fail(r, "expected a hexadecimal digit");
}

void zo_put_string(smart_str *out, const zend_string *s)
{
    smart_str_appendc(out, '"');
    for (size_t i = 0; i < ZSTR_LEN(s); ++i) {
        unsigned char c = (unsigned char)ZSTR_VAL(s)[i];
        if (c < 0x20 || c > 0x7e) {
            char escape[] = {'\\', 'x', hex_digits[c >> 4], hex_digits[c & 15]};
            smart_str_appendl(out, escape, sizeof(escape));
        } else {
            if (c == '"' || c == '\\') {
                smart_str_appendc(out, '\\');
            }
            smart_str_appendc(out, (char)c);
        }
    }
    smart_str_appendc(out, '"');
}

zend_string *zo_string(zo_reader *r)
{
    smart_str decoded = {0};
    zo_expect(r, "\"");
    while (r->p < r->end) {
        unsigned char c = (unsigned char)*r->p++;
        if (c == '"') {
            zend_string *string = smart_str_extract(&decoded);
            zend_string_hash_val(string);
            return string;
        }
        if (c < 0x20 || c > 0x7e) {
            zo_fail(r, "unescaped byte in quoted string");
        }
        if (c == '\\') {
            if (r->p == r->end) {
                break;
            }
            c = (unsigned char)*r->p++;
            if (c == 'x') {
                unsigned high = read_hex(r);
                c = (unsigned char)((high << 4) | read_hex(r));
            } else if (c != '\\' && c != '"') {
                zo_fail(r, "unknown string escape");
            }
        }
        smart_str_appendc(&decoded, (char)c);
    }
    zo_fail(r, "unterminated quoted string");
}

static bool nan_bits(uint64_t bits)
{
    return (bits & UINT64_C(0x7fffffffffffffff)) > UINT64_C(0x7ff0000000000000);
}

static void print_float(smart_str *out, double value)
{
    uint64_t bits;
    ZEND_STATIC_ASSERT(sizeof(bits) == sizeof(value), "binary64 required");
    memcpy(&bits, &value, sizeof(bits));
    smart_str_appends(out, "float(");
    if (nan_bits(bits)) {
        smart_str_appends(out, "nan:0x");
        for (int shift = 60; shift >= 0; shift -= 4) {
            smart_str_appendc(out, hex_digits[(bits >> shift) & 15]);
        }
    } else if (zend_finite(value)) {
        smart_str_append_double(out, value, -1, false);
    } else {
        smart_str_appends(out, value < 0 ? "-inf" : "inf");
    }
    smart_str_appendc(out, ')');
}

static double read_float(zo_reader *r)
{
    if (zo_take(r, "nan:0x")) {
        uint64_t bits = 0;
        for (unsigned i = 0; i < 16; ++i) {
            bits = (bits << 4) | read_hex(r);
        }
        if (!nan_bits(bits)) {
            zo_fail(r, "invalid NaN bit pattern");
        }
        double value;
        memcpy(&value, &bits, sizeof(value));
        return value;
    }
    if (zo_take(r, "-inf")) return -INFINITY;
    if (zo_take(r, "inf")) return INFINITY;

    const char *start = r->p;
    while (r->p < r->end && (decimal_digit(*r->p) || *r->p == '+' || *r->p == '-'
            || *r->p == '.' || *r->p == 'e' || *r->p == 'E')) {
        ++r->p;
    }
    zend_string *number = zend_string_init(start, (size_t)(r->p - start), false);
    const char *end;
    double value = zend_strtod(ZSTR_VAL(number), &end);
    bool valid = ZSTR_LEN(number) && end == ZSTR_VAL(number) + ZSTR_LEN(number) && zend_finite(value);
    zend_string_release(number);
    if (!valid) {
        zo_fail(r, "expected a finite decimal float, infinity or a NaN bit pattern");
    }
    return value;
}

#define ZO_MAX_VALUE_DEPTH 256

static void put_value(zo_writer *writer, const zval *value, unsigned depth);
static bool get_value(zo_reader *r, zval *value, unsigned depth);

static void print_array(zo_writer *writer, const HashTable *table, unsigned depth)
{
    smart_str *out = writer->out;
    zend_ulong index;
    zend_string *key;
    zval *entry;
    uint32_t count = 0, cursor = zend_hash_num_elements(table);
    const zval *current = zend_hash_get_current_data(table);
    zend_long next = ZEND_LONG_MIN;
    smart_str_appends(out, "array(");
    ZEND_HASH_FOREACH_KEY_VAL(table, index, key, entry) {
        if (entry == current) cursor = count;
        if (count++) smart_str_appends(out, ", ");
        if (key) {
            zo_put_string(out, key);
        } else {
            zend_long n = (zend_long)index;
            smart_str_append_long(out, n);
            if (n >= next) next = n == ZEND_LONG_MAX ? n : n + 1;
        }
        smart_str_appends(out, " => ");
        put_value(writer, entry, depth + 1);
    } ZEND_HASH_FOREACH_END();
    if (next != table->nNextFreeElement) {
        smart_str_appends(out, "; next=");
        smart_str_append_long(out, table->nNextFreeElement);
    }
    if (cursor != 0) {
        smart_str_appends(out, "; cursor=");
        smart_str_append_unsigned(out, cursor);
    }
    smart_str_appendc(out, ')');
}

void zo_put_table(zo_writer *writer, const HashTable *table)
{
    print_array(writer, table, 0);
}

static void put_array_value(zo_writer *writer, const HashTable *table, unsigned depth)
{
    zend_ulong pointer = (zend_ulong)(uintptr_t)table;
    zval *first = zend_hash_index_find(writer->array_offsets, pointer);
    if (first) {
        if (Z_TYPE_P(first) != IS_LONG) zo_unsupported("recursive literal array");
        zend_ulong offset = Z_LVAL_P(first);
        zval *label = zend_hash_index_find(writer->array_labels, offset);
        if (!label) {
            zval id;
            ZVAL_LONG(&id, zend_hash_num_elements(writer->array_labels));
            label = zend_hash_index_add_new(writer->array_labels, offset, &id);
        }
        smart_str_append_printf(writer->out, "array-ref(" ZEND_LONG_FMT ")", Z_LVAL_P(label));
        return;
    }
    zval position;
    ZVAL_LONG(&position, smart_str_get_len(writer->out) + sizeof("array") - 1);
    zend_hash_index_add_empty_element(writer->array_offsets, pointer);
    print_array(writer, table, depth);
    zend_hash_index_update(writer->array_offsets, pointer, &position);
}

void zo_insert_array_labels(zo_writer *writer)
{
    if (!zend_hash_num_elements(writer->array_labels)) return;
    smart_str labeled = {0};
    zend_string *original = writer->out->s;
    size_t copied = 0;
    zval *first;
    /* First occurrences are in output order. Insert only the labels used by a later value. */
    ZEND_HASH_FOREACH_VAL(writer->array_offsets, first) {
        size_t offset = (size_t)Z_LVAL_P(first);
        zval *label = zend_hash_index_find(writer->array_labels, offset);
        if (label) {
            smart_str_appendl(&labeled, ZSTR_VAL(original) + copied, offset - copied);
            smart_str_appendc(&labeled, '#');
            smart_str_append_long(&labeled, Z_LVAL_P(label));
            copied = offset;
        }
    } ZEND_HASH_FOREACH_END();
    smart_str_appendl(&labeled, ZSTR_VAL(original) + copied, ZSTR_LEN(original) - copied);
    smart_str_free(writer->out);
    *writer->out = labeled;
}

/* The result tells a shared binding whether it can also serve as an immutable partial argument. */
static bool read_array(zo_reader *r, zval *result, unsigned depth)
{
    array_init(result);
    HashTable *table = Z_ARRVAL_P(result);
    bool freezable = true;
    skip_space(r);
    if (r->p < r->end && *r->p != ')' && *r->p != ';') {
        do {
            skip_space(r);
            zend_string *key = NULL;
            zend_long index = 0;
            if (r->p < r->end && *r->p == '"') {
                key = zo_string(r);
            } else {
                index = zo_long(r);
            }
            zo_expect(r, "=>");
            zval item;
            if (!get_value(r, &item, depth + 1)) freezable = false;
            if (Z_ISUNDEF(item)) {
                zo_fail(r, "undef cannot be an array element");
            }
            zval *added = key ? zend_hash_add(table, key, &item)
                              : zend_hash_index_add(table, (zend_ulong)index, &item);
            if (key) zend_string_release(key);
            if (!added) zo_fail(r, "duplicate array key");
        } while (zo_take(r, ","));
    }
    if (zo_take(r, ";")) {
        if (zo_take(r, "next=")) {
            zend_long next = zo_long(r);
            if (next < table->nNextFreeElement) {
                zo_fail(r, "next array index precedes an existing integer key");
            }
            table->nNextFreeElement = next;
            if (zo_take(r, ")")) return freezable;
            zo_expect(r, ";");
        }
        zo_expect(r, "cursor=");
        uint32_t cursor = zo_u32(r);
        if (cursor > zend_hash_num_elements(table)) {
            zo_fail(r, "array cursor is outside the entries");
        }
        for (uint32_t i = 0; i < cursor; ++i) {
            zend_hash_move_forward(table);
        }
    }
    zo_expect(r, ")");
    return freezable;
}

static void put_value(zo_writer *writer, const zval *value, unsigned depth)
{
    smart_str *out = writer->out;
    if (depth > ZO_MAX_VALUE_DEPTH) zo_unsupported("literal nesting exceeds %d", ZO_MAX_VALUE_DEPTH);
    switch (Z_TYPE_P(value)) {
        case IS_UNDEF: smart_str_appends(out, "undef"); break;
        case IS_NULL:  smart_str_appends(out, "null"); break;
        case IS_FALSE: smart_str_appends(out, "false"); break;
        case IS_TRUE:  smart_str_appends(out, "true"); break;
        case IS_LONG:
            smart_str_appends(out, "int(");
            smart_str_append_long(out, Z_LVAL_P(value));
            smart_str_appendc(out, ')');
            break;
        case IS_DOUBLE: print_float(out, Z_DVAL_P(value)); break;
        case IS_STRING: zo_put_string(out, Z_STR_P(value)); break;
        case IS_ARRAY:  put_array_value(writer, Z_ARRVAL_P(value), depth); break;
        case IS_CONSTANT_AST: zo_put_ast(writer, value); break;
        default: zo_unsupported("literal with Zend type %d", Z_TYPE_P(value));
    }
}

static bool get_value(zo_reader *r, zval *value, unsigned depth)
{
    if (depth > ZO_MAX_VALUE_DEPTH) zo_fail(r, "literal nesting exceeds %d", ZO_MAX_VALUE_DEPTH);
    Z_EXTRA_P(value) = 0;
    if (zo_take(r, "undef")) { ZVAL_UNDEF(value); return true; }
    if (zo_take(r, "null"))  { ZVAL_NULL(value); return true; }
    if (zo_take(r, "false")) { ZVAL_FALSE(value); return true; }
    if (zo_take(r, "true"))  { ZVAL_TRUE(value); return true; }
    if (zo_take(r, "int(")) {
        ZVAL_LONG(value, zo_long(r));
        zo_expect(r, ")");
    } else if (zo_take(r, "float(")) {
        ZVAL_DOUBLE(value, read_float(r));
        zo_expect(r, ")");
    } else if (r->p < r->end && *r->p == '"') {
        ZVAL_STR(value, zo_string(r));
    } else if (zo_take(r, "array(")) {
        return read_array(r, value, depth);
    } else if (zo_take(r, "array#")) {
        uint32_t id = zo_u32(r);
        zo_expect(r, "(");
        if (!zend_hash_index_add_empty_element(r->array_bindings, id)) {
            zo_fail(r, "duplicate literal array binding %u", id);
        }
        bool freezable = read_array(r, value, depth);
        if (freezable) zo_freeze_bound_literal(r, value);
        Z_TRY_ADDREF_P(value);
        zend_hash_index_update(r->array_bindings, id, value);
        return freezable;
    } else if (zo_take(r, "array-ref(")) {
        uint32_t id = zo_u32(r);
        zo_expect(r, ")");
        zval *bound = zend_hash_index_find(r->array_bindings, id);
        if (!bound || Z_TYPE_P(bound) != IS_ARRAY) {
            zo_fail(r, "expected a completed literal array binding %u", id);
        }
        if (r->in_ast && Z_REFCOUNTED_P(bound)) {
            zo_fail(r, "a constant-expression value cannot contain a deferred zval");
        }
        ZVAL_COPY(value, bound);
        return !Z_REFCOUNTED_P(bound);
    } else if (zo_take(r, "ast")) {
        zo_get_ast(r, value);
        return false;
    } else {
        zo_fail(r, "expected a literal value");
    }
    return true;
}

void zo_put_value(zo_writer *writer, const zval *value)
{
    put_value(writer, value, 0);
}

void zo_get_value(zo_reader *r, zval *value)
{
    get_value(r, value, 0);
}
