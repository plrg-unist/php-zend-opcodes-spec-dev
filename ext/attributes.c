#include "opcodes.h"
#include "Zend/zend_attributes.h"
#include "Zend/zend_multiply.h"

void zo_put_attributes(smart_str *out, const HashTable *attrs)
{
    if (!attrs) {
        return;
    }
    zend_attribute *attr;
    ZEND_HASH_FOREACH_PTR(attrs, attr) {
        smart_str_appends(out, "attribute ");
        zo_put_string(out, attr->name);
        smart_str_append_printf(out, " %u %u %u %u ", attr->offset, attr->lineno,
            attr->flags & ~ZEND_ATTRIBUTE_PERSISTENT, attr->argc);
        if (attr->validation_error) {
            zo_put_string(out, attr->validation_error);
        } else {
            smart_str_appends(out, "none");
        }
        smart_str_appendc(out, '\n');
        for (uint32_t i = 0; i < attr->argc; i++) {
            smart_str_appends(out, "attrarg ");
            if (attr->args[i].name) {
                zo_put_string(out, attr->args[i].name);
            } else {
                smart_str_appends(out, "none");
            }
            smart_str_appendc(out, ' ');
            zo_put_value(out, &attr->args[i].value);
            smart_str_appendc(out, '\n');
        }
    } ZEND_HASH_FOREACH_END();
}

static zend_string *attribute_name(zo_reader *r)
{
    zend_string *name = zo_string(r);
    if (!ZSTR_LEN(name) || memchr(ZSTR_VAL(name), 0, ZSTR_LEN(name))) {
        zo_fail(r, "expected a nonempty attribute or argument name without NUL bytes");
    }
    return name;
}

HashTable *zo_get_attributes(zo_reader *r, uint32_t max_offset)
{
    HashTable *attrs = NULL;
    while (zo_take(r, "attribute")) {
        zend_string *name = attribute_name(r);
        uint32_t offset = zo_u32(r);
        uint32_t line = zo_u32(r);
        uint32_t flags = zo_u32(r);
        uint32_t argc = zo_u32(r);
        if (offset > max_offset || (flags & ~ZEND_ATTRIBUTE_STRICT_TYPES)) {
            zo_fail(r, "invalid attribute offset or flags");
        }
        bool overflow;
        zend_safe_address(argc, sizeof(zend_attribute_arg), offsetof(zend_attribute, args), &overflow);
        if (overflow || argc > (size_t)(r->end - r->p) / sizeof("attrarg")) {
            zo_fail(r, "attribute argument count exceeds the input");
        }
        zend_attribute *attr = zend_add_attribute(&attrs, name, argc, flags, offset, line);
        zend_string_release(name);
        if (!zo_take(r, "none")) {
            attr->validation_error = zo_string(r);
        }
        zo_eol(r);
        HashTable *names = NULL;
        for (uint32_t i = 0; i < argc; i++) {
            zo_expect(r, "attrarg");
            if (zo_take(r, "none")) {
                if (names) {
                    zo_fail(r, "positional attribute argument follows a named argument");
                }
            } else {
                attr->args[i].name = attribute_name(r);
                if (!names) {
                    names = zend_new_array(0);
                }
                if (!zend_hash_add_empty_element(names, attr->args[i].name)) {
                    zo_fail(r, "duplicate named attribute argument");
                }
            }
            zo_get_value(r, &attr->args[i].value);
            if (Z_ISUNDEF(attr->args[i].value)) {
                zo_fail(r, "undefined attribute argument");
            }
            zo_eol(r);
        }
        if (names) {
            zend_array_destroy(names);
        }
    }
    return attrs;
}
