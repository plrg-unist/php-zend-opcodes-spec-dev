#include "opcodes.h"
#include "Zend/zend_enum.h"

#define CASE_FLAGS (ZEND_ACC_DEPRECATED | ZEND_ACC_OVERRIDE)

static bool generated_property(const zend_class_entry *ce, zend_string *name, uint32_t slot,
    uint32_t type_mask)
{
    const zend_property_info *prop = zend_hash_find_ptr(&ce->properties_info, name);
    return prop && prop->ce == ce && prop->offset == OBJ_PROP_TO_OFFSET(slot)
        && prop->flags == (ZEND_ACC_PUBLIC | ZEND_ACC_READONLY | ZEND_ACC_PROTECTED_SET)
        && ZEND_TYPE_FULL_MASK(prop->type) == type_mask && !prop->attributes && !prop->doc_comment
        && !prop->hooks && Z_ISUNDEF(ce->default_properties_table[slot]);
}

void zo_put_enum(smart_str *out, const zend_class_entry *ce)
{
    bool backed = ce->enum_backing_type != IS_UNDEF;
    unsigned count = backed ? 2 : 1;
    if (zend_hash_num_elements(&ce->properties_info) != count ||
        ce->default_properties_count != (int)count || ce->default_static_members_count ||
        !generated_property(ce, ZSTR_KNOWN(ZEND_STR_NAME), 0, MAY_BE_STRING) ||
        (backed && !generated_property(ce, ZSTR_KNOWN(ZEND_STR_VALUE), 1,
            ce->enum_backing_type == IS_LONG ? MAY_BE_LONG : MAY_BE_STRING))) {
        zo_unsupported("unexpected properties on enum %s", ZSTR_VAL(ce->name));
    }
    switch (ce->enum_backing_type) {
        case IS_UNDEF: smart_str_appends(out, "enum pure\n"); break;
        case IS_LONG: smart_str_appends(out, "enum int\n"); break;
        case IS_STRING: smart_str_appends(out, "enum string\n"); break;
        default: zo_unsupported("enum backing type on %s", ZSTR_VAL(ce->name));
    }
}

static void declare_enum_property(zend_class_entry *ce, zend_string *name, uint32_t code)
{
    zval value;
    ZVAL_UNDEF(&value);
    zend_type type = ZEND_TYPE_INIT_MASK(code == IS_LONG ? MAY_BE_LONG : MAY_BE_STRING);
    zend_declare_typed_property(ce, name, &value, ZEND_ACC_PUBLIC | ZEND_ACC_READONLY, NULL, type);
}

void zo_get_enum(zo_reader *r, zend_class_entry *ce)
{
    zo_expect(r, "enum");
    if (zo_take(r, "pure")) {
        ce->enum_backing_type = IS_UNDEF;
    } else if (zo_take(r, "int")) {
        ce->enum_backing_type = IS_LONG;
    } else {
        zo_expect(r, "string");
        ce->enum_backing_type = IS_STRING;
    }
    zo_eol(r);
    ce->default_object_handlers = &zend_enum_object_handlers;
    declare_enum_property(ce, ZSTR_KNOWN(ZEND_STR_NAME), IS_STRING);
    if (ce->enum_backing_type != IS_UNDEF) {
        declare_enum_property(ce, ZSTR_KNOWN(ZEND_STR_VALUE), ce->enum_backing_type);
    }
}

void zo_check_enum(zo_reader *r, const zend_class_entry *ce)
{
    bool backed = ce->enum_backing_type != IS_UNDEF;
    uint32_t required = ZEND_ACC_ENUM | ZEND_ACC_FINAL | ZEND_ACC_NO_DYNAMIC_PROPERTIES;
    uint32_t forbidden = ZEND_ACC_INTERFACE | ZEND_ACC_TRAIT | ZEND_ACC_ANON_CLASS |
        ZEND_ACC_IMPLICIT_ABSTRACT_CLASS | ZEND_ACC_EXPLICIT_ABSTRACT_CLASS | ZEND_ACC_READONLY_CLASS;
    if ((ce->ce_flags & required) != required || (ce->ce_flags & forbidden) || ce->parent_name ||
        zend_hash_num_elements(&ce->properties_info) != (backed ? 2u : 1u)) {
        zo_fail(r, "invalid enum declaration for %s", ZSTR_VAL(ce->name));
    }
    unsigned unit_count = 0, backed_count = 0;
    for (uint32_t i = 0; i < ce->num_interfaces; i++) {
        zend_string *name = ce->interface_names[i].lc_name;
        unit_count += zend_string_equals_literal(name, "unitenum");
        backed_count += zend_string_equals_literal(name, "backedenum");
    }
    if (unit_count != 1 || backed_count != (unsigned)backed) {
        zo_fail(r, "invalid enum interfaces for %s", ZSTR_VAL(ce->name));
    }
}

void zo_put_enum_case(zo_writer *writer, zend_string *name, const zend_class_constant *constant)
{
    smart_str *out = writer->out;
    uint32_t flags = ZEND_CLASS_CONST_FLAGS(constant) & ~CONST_OWNED;
    if (Z_TYPE(constant->value) != IS_CONSTANT_AST ||
        (flags & ~CASE_FLAGS) != (ZEND_ACC_PUBLIC | ZEND_CLASS_CONST_IS_CASE)) {
        zo_unsupported("enum case representation for %s::%s", ZSTR_VAL(constant->ce->name), ZSTR_VAL(name));
    }
    zend_ast *init = Z_ASTVAL(constant->value);
    if (init->kind != ZEND_AST_CONST_ENUM_INIT || init->attr) {
        zo_unsupported("enum case initializer for %s", ZSTR_VAL(name));
    }
    for (unsigned int i = 0; i < 3; i++) {
        if (!init->child[i] || init->child[i]->kind != ZEND_AST_ZVAL || init->child[i]->attr ||
            zend_ast_get_lineno(init->child[i]) != init->lineno) {
            zo_unsupported("enum case metadata for %s", ZSTR_VAL(name));
        }
    }
    if (Z_TYPE_P(zend_ast_get_zval(init->child[0])) != IS_STRING ||
        !zend_string_equals(zend_ast_get_str(init->child[0]), constant->ce->name) ||
        Z_TYPE_P(zend_ast_get_zval(init->child[1])) != IS_LONG ||
        Z_TYPE_P(zend_ast_get_zval(init->child[2])) != IS_STRING ||
        !zend_string_equals(zend_ast_get_str(init->child[2]), name)) {
        zo_unsupported("enum case identity for %s", ZSTR_VAL(name));
    }
    smart_str_append_printf(out, "case " ZEND_LONG_FMT " ", Z_LVAL_P(zend_ast_get_zval(init->child[1])));
    zo_put_string(out, name);
    smart_str_append_printf(out, " %u %u ", flags & CASE_FLAGS, init->lineno);
    if (constant->doc_comment) {
        zo_put_string(out, constant->doc_comment);
    } else {
        smart_str_appendc(out, '-');
    }
    smart_str_appends(out, " = ");
    zo_put_expression(writer, init->child[3]);
    smart_str_appendc(out, '\n');
    zo_put_attributes(writer, constant->attributes);
}

void zo_get_enum_case(zo_reader *r, zend_class_entry *ce, uint32_t expected_id)
{
    uint32_t id = zo_u32(r);
    zend_string *name = zend_new_interned_string(zo_string(r));
    uint32_t flags = zo_u32(r), line = zo_u32(r);
    if (!(ce->ce_flags & ZEND_ACC_ENUM) || id != expected_id || id > INT_MAX ||
        !ZSTR_LEN(name) || memchr(ZSTR_VAL(name), 0, ZSTR_LEN(name)) ||
        (flags & ~CASE_FLAGS) || zend_hash_exists(&ce->constants_table, name)) {
        zo_fail(r, "invalid enum case id, name or flags");
    }
    zend_string *doc = zo_take(r, "-") ? NULL : zo_string(r);
    zo_expect(r, "=");
    zend_ast_ref *backing = zo_get_expression(r, false);
    if ((backing != NULL) != (ce->enum_backing_type != IS_UNDEF)) {
        zo_fail(r, "enum case %s has an inconsistent backing expression", ZSTR_VAL(name));
    }
    zo_eol(r);

    zend_ast_zval leaves[3] = {0};
    ZVAL_STR(&leaves[0].val, ce->name);
    ZVAL_LONG(&leaves[1].val, id);
    ZVAL_STR(&leaves[2].val, name);
    zend_ast *init = emalloc(zend_ast_size(4));
    init->kind = ZEND_AST_CONST_ENUM_INIT;
    init->attr = 0;
    init->lineno = line;
    for (unsigned int i = 0; i < 3; i++) {
        leaves[i].kind = ZEND_AST_ZVAL;
        Z_LINENO(leaves[i].val) = line;
        init->child[i] = (zend_ast *)&leaves[i];
    }
    init->child[3] = backing ? GC_AST(backing) : NULL;
    zval value;
    ZVAL_AST(&value, zend_ast_copy(init));
    efree(init);
    if (backing) zend_ast_ref_destroy(backing);
    zend_class_constant *constant = zend_declare_class_constant_ex(ce, name, &value,
        ZEND_ACC_PUBLIC | ZEND_CLASS_CONST_IS_CASE | flags, doc);
    constant->attributes = zo_get_attributes(r, 0);
    zend_string_release(name);
}
