#include "opcodes.h"
#include "Zend/zend_constants.h"
#include "Zend/zend_inheritance.h"
#include "Zend/zend_observer.h"
#include "Zend/zend_property_hooks.h"

#define CLASS_FLAGS (ZEND_ACC_INTERFACE | ZEND_ACC_TRAIT | ZEND_ACC_ANON_CLASS | ZEND_ACC_ENUM | \
    ZEND_ACC_IMPLICIT_ABSTRACT_CLASS | ZEND_ACC_FINAL | ZEND_ACC_EXPLICIT_ABSTRACT_CLASS | \
    ZEND_ACC_TOP_LEVEL | ZEND_ACC_DEPRECATED | ZEND_ACC_NO_DYNAMIC_PROPERTIES | \
    ZEND_ACC_ALLOW_DYNAMIC_PROPERTIES | ZEND_ACC_READONLY_CLASS | ZEND_HAS_STATIC_IN_METHODS | \
    ZEND_ACC_NOT_SERIALIZABLE | ZEND_ACC_USE_GUARDS)
#define PROPERTY_FLAGS (ZEND_ACC_PPP_MASK | ZEND_ACC_PPP_SET_MASK | ZEND_ACC_STATIC | \
    ZEND_ACC_FINAL | ZEND_ACC_ABSTRACT | ZEND_ACC_READONLY | ZEND_ACC_PROMOTED | \
    ZEND_ACC_VIRTUAL | ZEND_ACC_DEPRECATED | ZEND_ACC_OVERRIDE | ZEND_ACC_CHANGED)
#define CONSTANT_FLAGS (ZEND_ACC_PPP_MASK | ZEND_ACC_FINAL | ZEND_ACC_DEPRECATED | ZEND_ACC_OVERRIDE)

typedef struct {
    zend_class_entry ce;
    bool link;
} unit_class;

static void put_optional_string(smart_str *out, const zend_string *s)
{
    if (s) {
        zo_put_string(out, s);
    } else {
        smart_str_appendc(out, '-');
    }
}

static zend_string *optional_string(zo_reader *r)
{
    return zo_take(r, "-") ? NULL : zo_string(r);
}

static zend_string *halt_name(const zend_string *filename)
{
    return zend_mangle_property_name(ZEND_STRL("__COMPILER_HALT_OFFSET__"),
        ZSTR_VAL(filename), ZSTR_LEN(filename), false);
}

static const zend_known_string_id autoglobals[] = {
    ZEND_STR_AUTOGLOBAL_SERVER, ZEND_STR_AUTOGLOBAL_ENV, ZEND_STR_AUTOGLOBAL_REQUEST
};

static int autoglobal_index(const zend_string *name)
{
    for (size_t i = 0; i < sizeof(autoglobals) / sizeof(*autoglobals); i++) {
        if (zend_string_equals(name, ZSTR_KNOWN(autoglobals[i]))) {
            return (int)i;
        }
    }
    return -1;
}

uint8_t zo_armed_autoglobals(void)
{
    uint8_t mask = 0;
    for (size_t i = 0; i < sizeof(autoglobals) / sizeof(*autoglobals); i++) {
        zend_auto_global *global = zend_hash_find_ptr(CG(auto_globals), ZSTR_KNOWN(autoglobals[i]));
        if (global && global->jit && global->armed) {
            mask |= 1u << i;
        }
    }
    return mask;
}

static void print_autoglobals(smart_str *out, uint8_t before)
{
    uint8_t added = before & ~zo_armed_autoglobals();
    if (!added) {
        return;
    }
    /* The order in which compilation inserted these names is visible in $GLOBALS. */
    zend_string *name;
    ZEND_HASH_FOREACH_STR_KEY(&EG(symbol_table), name) {
        int i = name ? autoglobal_index(name) : -1;
        if (i >= 0 && (added & (1u << i))) {
            smart_str_appends(out, "autoglobal ");
            zo_put_string(out, name);
            smart_str_appendc(out, '\n');
        }
    } ZEND_HASH_FOREACH_END();
}

static void print_property(zo_writer *writer, const zend_class_entry *ce, zend_string *key,
    const zend_property_info *prop)
{
    smart_str *out = writer->out;
    zval value;
    if (prop->flags & ~PROPERTY_FLAGS) {
        zo_unsupported("property flags on %s::$%s", ZSTR_VAL(ce->name), ZSTR_VAL(key));
    }
    if (prop->offset == ZEND_VIRTUAL_PROPERTY_OFFSET) {
        ZVAL_UNDEF(&value);
    } else if (prop->flags & ZEND_ACC_STATIC) {
        ZVAL_COPY_VALUE(&value, &ce->default_static_members_table[prop->offset]);
    } else {
        ZVAL_COPY_VALUE(&value, &ce->default_properties_table[OBJ_PROP_TO_NUM(prop->offset)]);
    }
    smart_str_appends(out, "property ");
    zo_put_string(out, key);
    smart_str_append_printf(out, " %u ", prop->flags);
    zo_put_type(out, prop->type);
    smart_str_appendc(out, ' ');
    put_optional_string(out, prop->doc_comment);
    smart_str_appends(out, " = ");
    zo_put_value(writer, &value);
    smart_str_appendc(out, '\n');
    zo_put_attributes(writer, prop->attributes);
    if (prop->hooks) {
        for (uint32_t i = 0; i < ZEND_PROPERTY_HOOK_COUNT; i++) {
            zend_function *hook = prop->hooks[i];
            if (hook && hook->common.scope == ce) {
                smart_str_appends(out, i == ZEND_PROPERTY_HOOK_GET ? "get\n" : "set\n");
                zo_print_body(writer, &hook->op_array);
            }
        }
    }
}

static void print_class(zo_writer *writer, zend_string *key, const zend_class_entry *ce)
{
    smart_str *out = writer->out;
    if (ce->ce_flags2 || (ce->num_traits && ce->ce_flags & ZEND_ACC_LINKED)) {
        zo_unsupported("linked trait composition or class flags on %s", ZSTR_VAL(ce->name));
    }
    zend_class_entry *parent = ce->ce_flags & ZEND_ACC_RESOLVED_PARENT ? ce->parent : NULL;
    smart_str_appends(out, "class ");
    zo_put_string(out, key);
    smart_str_appendc(out, ' ');
    zo_put_string(out, ce->name);
    smart_str_append_printf(out, " %s %u %u %u ",
        ce->ce_flags & ZEND_ACC_LINKED ? "linked" : "unlinked",
        ce->ce_flags & CLASS_FLAGS, ce->info.user.line_start, ce->info.user.line_end);
    put_optional_string(out, parent ? parent->name : ce->parent_name);
    smart_str_appendc(out, ' ');
    put_optional_string(out, ce->doc_comment);
    smart_str_appends(out, " {\n");
    zo_put_attributes(writer, ce->attributes);
    if (ce->ce_flags & ZEND_ACC_ENUM) {
        zo_put_enum(out, ce);
    }
    uint32_t inherited = (ce->ce_flags & ZEND_ACC_RESOLVED_INTERFACES) && parent
        ? parent->num_interfaces : 0;
    for (uint32_t i = inherited; i < ce->num_interfaces; i++) {
        zend_string *name = ce->ce_flags & ZEND_ACC_RESOLVED_INTERFACES
            ? ce->interfaces[i]->name : ce->interface_names[i].name;
        smart_str_appends(out, "implements ");
        zo_put_string(out, name);
        smart_str_appendc(out, '\n');
    }
    for (uint32_t i = 0; i < ce->num_traits; i++) {
        smart_str_appends(out, "use ");
        zo_put_string(out, ce->trait_names[i].name);
        smart_str_appendc(out, '\n');
    }
    for (uint32_t i = 0; ce->trait_aliases && ce->trait_aliases[i]; i++) {
        zend_trait_alias *alias = ce->trait_aliases[i];
        smart_str_appends(out, "alias ");
        put_optional_string(out, alias->trait_method.class_name);
        smart_str_appendc(out, ' ');
        zo_put_string(out, alias->trait_method.method_name);
        smart_str_append_printf(out, " %u ", alias->modifiers);
        put_optional_string(out, alias->alias);
        smart_str_appendc(out, '\n');
    }
    for (uint32_t i = 0; ce->trait_precedences && ce->trait_precedences[i]; i++) {
        zend_trait_precedence *p = ce->trait_precedences[i];
        smart_str_appends(out, "instead ");
        zo_put_string(out, p->trait_method.class_name);
        smart_str_appendc(out, ' ');
        zo_put_string(out, p->trait_method.method_name);
        smart_str_append_printf(out, " %u", p->num_excludes);
        for (uint32_t j = 0; j < p->num_excludes; j++) {
            smart_str_appendc(out, ' ');
            zo_put_string(out, p->exclude_class_names[j]);
        }
        smart_str_appendc(out, '\n');
    }
    zend_property_info *prop;
    ZEND_HASH_MAP_FOREACH_STR_KEY_PTR(&ce->properties_info, key, prop) {
        if (prop->ce == ce && !(ce->ce_flags & ZEND_ACC_ENUM)) {
            print_property(writer, ce, key, prop);
        }
    } ZEND_HASH_FOREACH_END();
    zend_class_constant *constant;
    ZEND_HASH_MAP_FOREACH_STR_KEY_PTR(&ce->constants_table, key, constant) {
        if (constant->ce == ce) {
            if (ce->ce_flags & ZEND_ACC_ENUM && ZEND_CLASS_CONST_FLAGS(constant) & ZEND_CLASS_CONST_IS_CASE) {
                zo_put_enum_case(writer, key, constant);
                continue;
            }
            uint32_t flags = ZEND_CLASS_CONST_FLAGS(constant) & ~CONST_OWNED;
            if (flags & ~CONSTANT_FLAGS) {
                zo_unsupported("class constant flags on %s::%s", ZSTR_VAL(ce->name), ZSTR_VAL(key));
            }
            smart_str_appends(out, "constant ");
            zo_put_string(out, key);
            smart_str_append_printf(out, " %u ", flags);
            zo_put_type(out, constant->type);
            smart_str_appendc(out, ' ');
            put_optional_string(out, constant->doc_comment);
            smart_str_appends(out, " = ");
            zo_put_value(writer, &constant->value);
            smart_str_appendc(out, '\n');
            zo_put_attributes(writer, constant->attributes);
        }
    } ZEND_HASH_FOREACH_END();
    zend_function *fn;
    ZEND_HASH_MAP_FOREACH_STR_KEY_PTR(&ce->function_table, key, fn) {
        if (fn->common.scope == ce) {
            if (fn->type != ZEND_USER_FUNCTION) {
                if ((ce->ce_flags & ZEND_ACC_ENUM) && (zend_string_equals_literal(key, "cases")
                    || (ce->enum_backing_type != IS_UNDEF
                        && (zend_string_equals_literal(key, "from") || zend_string_equals_literal(key, "tryfrom"))))) {
                    continue;
                }
                zo_unsupported("internal method on %s", ZSTR_VAL(ce->name));
            }
            smart_str_appends(out, "method ");
            zo_put_string(out, key);
            smart_str_appendc(out, '\n');
            zo_print_body(writer, &fn->op_array);
        }
    } ZEND_HASH_FOREACH_END();
    smart_str_appends(out, "}\n");
}

static void print_unit(zo_writer *writer, const zend_op_array *main, uint32_t functions_start,
    uint32_t classes_start, uint8_t autoglobals_before)
{
    smart_str *out = writer->out;
    smart_str_appends(out, "zend-opcodes\nfile ");
    zo_put_string(out, main->filename);
    smart_str_appendc(out, '\n');
    zend_string *key = halt_name(main->filename);
    zend_constant *halt = zend_hash_find_ptr(EG(zend_constants), key);
    zend_string_release(key);
    if (halt) {
        smart_str_appends(out, "halt ");
        smart_str_append_long(out, Z_LVAL(halt->value));
        smart_str_appendc(out, '\n');
    }
    print_autoglobals(out, autoglobals_before);
    smart_str_appends(out, "main\n");
    zo_print_body(writer, main);
    Bucket *bucket;
    ZEND_HASH_MAP_FOREACH_BUCKET_FROM(CG(function_table), bucket, functions_start) {
        zend_function *fn = Z_PTR(bucket->val);
        if (fn->type == ZEND_USER_FUNCTION && zend_string_equals(fn->op_array.filename, main->filename)) {
            smart_str_appends(out, "function ");
            zo_put_string(out, bucket->key);
            smart_str_appendc(out, '\n');
            zo_print_body(writer, &fn->op_array);
        }
    } ZEND_HASH_FOREACH_END();
    ZEND_HASH_MAP_FOREACH_BUCKET_FROM(CG(class_table), bucket, classes_start) {
        zend_class_entry *ce = Z_PTR(bucket->val);
        if (ce->type == ZEND_USER_CLASS && zend_string_equals(ce->info.user.filename, main->filename)) {
            print_class(writer, bucket->key, ce);
        }
    } ZEND_HASH_FOREACH_END();
    smart_str_appends(out, "end\n");
}

void zo_print_unit(smart_str *out, const zend_op_array *main, uint32_t functions_start,
    uint32_t classes_start, uint8_t autoglobals_before)
{
    zo_writer writer = {out, zend_new_array(0), zend_new_array(0)};
    zend_try {
        print_unit(&writer, main, functions_start, classes_start, autoglobals_before);
        zo_insert_array_labels(&writer);
    } zend_catch {
        zend_array_release(writer.array_offsets);
        zend_array_release(writer.array_labels);
        smart_str_free(out);
        zend_bailout();
    } zend_end_try();
    zend_array_release(writer.array_offsets);
    zend_array_release(writer.array_labels);
}

static zend_string *read_name(zo_reader *r)
{
    zend_string *name = zo_string(r);
    if (!ZSTR_LEN(name) || memchr(ZSTR_VAL(name), 0, ZSTR_LEN(name))) {
        zo_fail(r, "expected a nonempty name without NUL bytes");
    }
    return zend_new_interned_string(name);
}

static bool is_lower_key(const zend_string *key, zend_string *name)
{
    zend_string *lc = zend_string_tolower(name);
    bool equal = zend_string_equals(key, lc);
    zend_string_release(lc);
    return equal;
}

static bool valid_class_key(zend_string *key, zend_string *name)
{
    if (!ZSTR_LEN(name) || !ZSTR_LEN(key)) {
        return false;
    }
    if (ZSTR_VAL(key)[0] != '\0') {
        return is_lower_key(key, name);
    }
    zend_string *lc = zend_string_tolower(name);
    bool valid = ZSTR_LEN(key) > ZSTR_LEN(lc) + 1
        && memcmp(ZSTR_VAL(key) + 1, ZSTR_VAL(lc), ZSTR_LEN(lc)) == 0;
    zend_string_release(lc);
    return valid;
}

static zend_op_array *read_function(zo_reader *r, zend_string *filename, zend_class_entry *scope)
{
    zend_op_array *body = zend_arena_alloc(&CG(arena), sizeof(*body));
    zo_load_body(r, body, filename, scope);
    if (!body->function_name || body->fn_flags & ZEND_ACC_CLOSURE || body->scope != scope) {
        zo_fail(r, "expected a named function with its declared scope");
    }
    return body;
}

static uint32_t member_flags(zo_reader *r, uint32_t allowed)
{
    uint32_t flags = zo_u32(r);
    uint32_t visibility = flags & ZEND_ACC_PPP_MASK;
    uint32_t set_visibility = flags & ZEND_ACC_PPP_SET_MASK;
    if ((flags & ~allowed) || !visibility || (visibility & (visibility - 1))
        || (set_visibility & (set_visibility - 1))) {
        zo_fail(r, "invalid declaration flags");
    }
    return flags;
}

static zend_type member_type(zo_reader *r)
{
    zend_type type = zo_get_type(r);
    if ((ZEND_TYPE_FULL_MASK(type) >> _ZEND_TYPE_EXTRA_FLAGS_SHIFT)
        || (ZEND_TYPE_PURE_MASK(type) & (MAY_BE_VOID | MAY_BE_NEVER | MAY_BE_CALLABLE))) {
        zo_fail(r, "invalid class member type");
    }
    return type;
}

static void load_property(zo_reader *r, zend_class_entry *ce)
{
    zend_string *name = read_name(r);
    uint32_t flags = member_flags(r, PROPERTY_FLAGS);
    zend_type type = member_type(r);
    zend_string *doc = optional_string(r);
    if (((flags & ZEND_ACC_READONLY) && (!ZEND_TYPE_IS_SET(type) || (flags & ZEND_ACC_STATIC)))
        || ((ce->ce_flags & ZEND_ACC_READONLY_CLASS) && !(flags & ZEND_ACC_READONLY))) {
        zo_fail(r, "invalid readonly property declaration");
    }
    if (zend_hash_exists(&ce->properties_info, name)) {
        zo_fail(r, "duplicate property %s", ZSTR_VAL(name));
    }
    zo_expect(r, "=");
    zval value;
    zo_get_value(r, &value);
    zo_eol(r);
    if (ZEND_TYPE_IS_SET(type) && !Z_ISUNDEF(value) && Z_TYPE(value) != IS_CONSTANT_AST
        && !ZEND_TYPE_CONTAINS_CODE(type, Z_TYPE(value))) {
        zo_fail(r, "default value does not match property type");
    }
    uint32_t declaration_flags = flags;
    if ((flags & (ZEND_ACC_PUBLIC | ZEND_ACC_READONLY | ZEND_ACC_PPP_SET_MASK))
        == (ZEND_ACC_PUBLIC | ZEND_ACC_READONLY)) {
        /* Zend strips explicit public(set) but defaults an omitted setter to protected(set). */
        declaration_flags |= ZEND_ACC_PUBLIC_SET;
    }
    zend_property_info *prop = zend_declare_typed_property(ce, name, &value, declaration_flags, doc, type);
    prop->attributes = zo_get_attributes(r, 0);
    for (uint32_t i = 0; i < ZEND_PROPERTY_HOOK_COUNT; i++) {
        const char *kind = i == ZEND_PROPERTY_HOOK_GET ? "get" : "set";
        if (!zo_take(r, kind)) {
            continue;
        }
        zo_eol(r);
        if (flags & (ZEND_ACC_STATIC | ZEND_ACC_READONLY)) {
            zo_fail(r, "a static or readonly property cannot have a hook");
        }
        zend_op_array *fn = read_function(r, ce->info.user.filename, ce);
        zend_string *hook_name = zend_strpprintf(0, "$%pS::%s", name, kind);
        if (!zend_string_equals(fn->function_name, hook_name)
            || fn->num_args != (i == ZEND_PROPERTY_HOOK_SET) || fn->fn_flags & ZEND_ACC_VARIADIC) {
            zo_fail(r, "invalid %s hook on property %s", kind, ZSTR_VAL(name));
        }
        zend_string_release(hook_name);
        if (!prop->hooks) {
            prop->hooks = zend_arena_calloc(&CG(arena), ZEND_PROPERTY_HOOK_COUNT, sizeof(zend_function *));
            ce->num_hooked_props++;
#ifndef ZEND_OPCACHE_SHM_REATTACHMENT
            ce->get_iterator = zend_hooked_object_get_iterator;
#endif
        }
        fn->prop_info = prop;
        prop->hooks[i] = (zend_function *)fn;
        if (i == ZEND_PROPERTY_HOOK_SET) {
            zend_inheritance_status status = zend_verify_property_hook_variance(prop, (zend_function *)fn);
            if (status == INHERITANCE_UNRESOLVED) {
                ce->num_hooked_prop_variance_checks++;
            } else if (status != INHERITANCE_SUCCESS) {
                zo_fail(r, "invalid setter type on property %s", ZSTR_VAL(name));
            }
        }
    }
    if ((flags & ZEND_ACC_VIRTUAL) && (!prop->hooks || !Z_ISUNDEF(value))) {
        zo_fail(r, "virtual property requires hooks and no default");
    }
    if ((flags & ZEND_ACC_ABSTRACT) && !prop->hooks) {
        zo_fail(r, "abstract property requires a hook");
    }
    if (prop->hooks && !ce->parent_name) {
        zend_verify_hooked_property(ce, prop, name);
    }
    zend_string_release(name);
}

static void load_constant(zo_reader *r, zend_class_entry *ce)
{
    zend_string *name = read_name(r);
    uint32_t flags = member_flags(r, CONSTANT_FLAGS);
    zend_type type = member_type(r);
    zend_string *doc = optional_string(r);
    zo_expect(r, "=");
    zval value;
    zo_get_value(r, &value);
    zo_eol(r);
    if (Z_ISUNDEF(value) || (ZEND_TYPE_IS_SET(type) && Z_TYPE(value) != IS_CONSTANT_AST
        && !ZEND_TYPE_CONTAINS_CODE(type, Z_TYPE(value)))) {
        zo_fail(r, "invalid value for class constant");
    }
    zend_class_constant *c = zend_declare_typed_class_constant(ce, name, &value, flags, doc, type);
    if (flags & (ZEND_ACC_OVERRIDE | ZEND_ACC_DEPRECATED)) {
        ce->ce_flags |= ZEND_ACC_HAS_AST_CONSTANTS;
        ce->ce_flags &= ~ZEND_ACC_CONSTANTS_UPDATED;
    }
    c->attributes = zo_get_attributes(r, 0);
    zend_string_release(name);
}

static void load_method(zo_reader *r, zend_class_entry *ce)
{
    zend_string *key = read_name(r);
    zo_eol(r);
    zend_op_array *body = read_function(r, ce->info.user.filename, ce);
    if (!is_lower_key(key, body->function_name) || !zend_hash_add_ptr(&ce->function_table, key, body)) {
        zo_fail(r, "invalid or duplicate method key %s", ZSTR_VAL(key));
    }
    zend_check_magic_method_implementation(ce, (zend_function *)body, key, E_COMPILE_ERROR);
    zend_add_magic_method(ce, (zend_function *)body, key);
    zend_string_release(key);
}

static void read_class_names(zo_reader *r, const char *token, zend_class_name **names, uint32_t *count)
{
    while (zo_take(r, token)) {
        zend_string *name = read_name(r);
        zo_eol(r);
        *names = safe_erealloc(*names, *count + 1, sizeof(zend_class_name), 0);
        (*names)[*count].name = name;
        (*names)[*count].lc_name = zend_string_tolower(name);
        (*count)++;
    }
}

static void read_trait_adaptations(zo_reader *r, zend_class_entry *ce)
{
    for (size_t i = 0; zo_take(r, "alias"); i++) {
        zend_trait_alias *alias = emalloc(sizeof(*alias));
        alias->trait_method.class_name = zo_take(r, "-") ? NULL : read_name(r);
        alias->trait_method.method_name = read_name(r);
        alias->modifiers = zo_u32(r);
        alias->alias = zo_take(r, "-") ? NULL : read_name(r);
        uint32_t visibility = alias->modifiers & ZEND_ACC_PPP_MASK;
        if (!ce->num_traits || (!alias->alias && !alias->modifiers)
            || (alias->modifiers & ~(ZEND_ACC_PPP_MASK | ZEND_ACC_FINAL))
            || (visibility & (visibility - 1))) {
            zo_fail(r, "invalid trait method alias");
        }
        zo_eol(r);
        ce->trait_aliases = safe_erealloc(ce->trait_aliases, i + 2, sizeof(*ce->trait_aliases), 0);
        ce->trait_aliases[i] = alias;
        ce->trait_aliases[i + 1] = NULL;
    }
    for (size_t i = 0; zo_take(r, "instead"); i++) {
        zend_string *class_name = read_name(r);
        zend_string *method = read_name(r);
        uint32_t n = zo_u32(r);
        if (!n || !ce->num_traits || n > (size_t)(r->end - r->p)) {
            zo_fail(r, "invalid trait precedence");
        }
        zend_trait_precedence *p = safe_emalloc(n, sizeof(zend_string *), offsetof(zend_trait_precedence, exclude_class_names));
        p->trait_method.class_name = class_name;
        p->trait_method.method_name = method;
        p->num_excludes = n;
        for (uint32_t j = 0; j < n; j++) {
            p->exclude_class_names[j] = read_name(r);
        }
        zo_eol(r);
        ce->trait_precedences = safe_erealloc(ce->trait_precedences, i + 2, sizeof(*ce->trait_precedences), 0);
        ce->trait_precedences[i] = p;
        ce->trait_precedences[i + 1] = NULL;
    }
}

static void load_class(zo_reader *r, zend_string *filename, HashTable *classes)
{
    zend_string *key = zend_new_interned_string(zo_string(r));
    unit_class *entry = zend_arena_calloc(&CG(arena), 1, sizeof(*entry));
    zend_class_entry *ce = &entry->ce;
    zend_class_entry *previous_scope = r->scope;
    r->scope = ce;
    ce->type = ZEND_USER_CLASS;
    ce->name = zend_new_interned_string(zo_string(r));
    zend_initialize_class_data(ce, true);
    ce->info.user.filename = zend_string_copy(filename);
    if (zo_take(r, "linked")) {
        entry->link = true;
    } else {
        zo_expect(r, "unlinked");
    }
    uint32_t flags = zo_u32(r);
    uint32_t kind = flags & (ZEND_ACC_INTERFACE | ZEND_ACC_TRAIT | ZEND_ACC_ENUM);
    if ((flags & ~CLASS_FLAGS) || (kind & (kind - 1))
        || (!(flags & ZEND_ACC_ANON_CLASS) && !zend_is_valid_class_name(ce->name))
        || !valid_class_key(key, ce->name)) {
        zo_fail(r, "invalid class name, declaration key or flags");
    }
    ce->ce_flags |= flags;
    if (!(flags & ZEND_ACC_ANON_CLASS)) {
        zend_alloc_ce_cache(ce->name);
    }
    ce->info.user.line_start = zo_u32(r);
    ce->info.user.line_end = zo_u32(r);
    if (ce->info.user.line_end < ce->info.user.line_start) {
        zo_fail(r, "invalid class source line range");
    }
    ce->parent_name = zo_take(r, "-") ? NULL : read_name(r);
    ce->doc_comment = optional_string(r);
    zo_expect(r, "{");
    zo_eol(r);
    ce->attributes = zo_get_attributes(r, 0);
    if (flags & ZEND_ACC_ENUM) {
        zo_get_enum(r, ce);
    }
    read_class_names(r, "implements", &ce->interface_names, &ce->num_interfaces);
    read_class_names(r, "use", &ce->trait_names, &ce->num_traits);
    if (ce->num_traits && (entry->link || flags & ZEND_ACC_INTERFACE)) {
        zo_fail(r, "only unlinked classes and traits can declare trait use");
    }
    read_trait_adaptations(r, ce);
    while (zo_take(r, "property")) {
        load_property(r, ce);
    }
    uint32_t next_case = 1;
    for (;;) {
        if (zo_take(r, "constant")) {
            load_constant(r, ce);
        } else if (flags & ZEND_ACC_ENUM && zo_take(r, "case")) {
            zo_get_enum_case(r, ce, next_case++);
        } else {
            break;
        }
    }
    while (zo_take(r, "method")) {
        load_method(r, ce);
    }
    zo_expect(r, "}");
    zo_eol(r);
    if (flags & ZEND_ACC_ENUM) {
        zo_check_enum(r, ce);
    }
    if (zend_hash_exists(CG(class_table), key) || !zend_hash_add_ptr(classes, key, entry)) {
        zo_fail(r, "duplicate class declaration");
    }
    r->scope = previous_scope;
    zend_string_release(key);
}

static bool dependency_linked(zend_string *name)
{
    zend_class_entry *ce = zend_hash_find_ptr_lc(CG(class_table), name);
    return ce && (ce->ce_flags & ZEND_ACC_LINKED);
}

static bool ready_to_link(const zend_class_entry *ce)
{
    if (ce->parent_name && !dependency_linked(ce->parent_name)) {
        return false;
    }
    for (uint32_t i = 0; i < ce->num_interfaces; i++) {
        if (!dependency_linked(ce->interface_names[i].name)) {
            return false;
        }
    }
    return true;
}

static void install_classes(zo_reader *r, HashTable *classes)
{
    zend_string *key;
    unit_class *entry;
    uint32_t pending = 0;
    ZEND_HASH_MAP_FOREACH_STR_KEY_PTR(classes, key, entry) {
        zend_hash_add_new_ptr(CG(class_table), key, &entry->ce);
        pending += entry->link;
    } ZEND_HASH_FOREACH_END();
    while (pending) {
        uint32_t before = pending;
        ZEND_HASH_MAP_FOREACH_STR_KEY_PTR(classes, key, entry) {
            zend_class_entry *ce = &entry->ce;
            if (!entry->link || !ready_to_link(ce)) {
                continue;
            }
            bool hidden = !ZSTR_VAL(key)[0];
            zend_class_entry *saved = hidden && ZSTR_HAS_CE_CACHE(ce->name) ? ZSTR_GET_CE_CACHE(ce->name) : NULL;
            zend_string *parent = ce->parent_name ? zend_string_tolower(ce->parent_name) : NULL;
            uint32_t abstract_flags = ce->ce_flags & (ZEND_ACC_EXPLICIT_ABSTRACT_CLASS | ZEND_ACC_IMPLICIT_ABSTRACT_CLASS);
            if (!zend_do_link_class(ce, parent, key)) {
                zo_fail(r, "cannot link class %s", ZSTR_VAL(ce->name));
            }
            /* Compiler early binding preserves this bit on explicitly abstract root classes. */
            if (abstract_flags == (ZEND_ACC_EXPLICIT_ABSTRACT_CLASS | ZEND_ACC_IMPLICIT_ABSTRACT_CLASS)) {
                ce->ce_flags |= ZEND_ACC_IMPLICIT_ABSTRACT_CLASS;
            }
            if (parent) {
                zend_string_release(parent);
            }
            if (hidden) {
                if (ZSTR_HAS_CE_CACHE(ce->name)) {
                    ZSTR_SET_CE_CACHE(ce->name, saved);
                }
            } else {
                zend_observer_class_linked_notify(ce, key);
            }
            entry->link = false;
            pending--;
        } ZEND_HASH_FOREACH_END();
        if (before == pending) {
            zo_fail(r, "linked class has a missing or circular dependency");
        }
    }
}

zend_op_array *zo_load_unit(zo_reader *r)
{
    zo_expect(r, "zend-opcodes");
    zo_eol(r);
    zo_expect(r, "file");
    zend_string *filename = read_name(r);
    r->filename = filename;
    zo_eol(r);
    zend_long halt_offset = -1;
    if (zo_take(r, "halt")) {
        halt_offset = zo_long(r);
        if (halt_offset < 0) {
            zo_fail(r, "invalid compiler halt offset");
        }
        zo_eol(r);
    }
    unsigned char order[sizeof(autoglobals) / sizeof(*autoglobals)];
    uint8_t used = 0;
    size_t count = 0;
    while (zo_take(r, "autoglobal")) {
        zend_string *name = read_name(r);
        int i = autoglobal_index(name);
        if (i < 0 || (used & (1u << i))) {
            zo_fail(r, "unknown or repeated autoglobal %s", ZSTR_VAL(name));
        }
        used |= 1u << i;
        order[count++] = (unsigned char)i;
        zend_string_release(name);
        zo_eol(r);
    }
    zo_expect(r, "main");
    zo_eol(r);
    zend_op_array *main = emalloc(sizeof(*main));
    zo_load_body(r, main, filename, NULL);
    if (main->function_name) {
        zo_fail(r, "main body has a function name");
    }
    HashTable functions, classes;
    zend_hash_init(&functions, 8, NULL, NULL, false);
    zend_hash_init(&classes, 8, NULL, NULL, false);
    while (zo_take(r, "function")) {
        zend_string *key = read_name(r);
        zo_eol(r);
        zend_op_array *body = read_function(r, filename, NULL);
        if (!is_lower_key(key, body->function_name) || zend_hash_exists(CG(function_table), key)
            || !zend_hash_add_ptr(&functions, key, body)) {
            zo_fail(r, "invalid or duplicate function key %s", ZSTR_VAL(key));
        }
        zend_string_release(key);
    }
    while (zo_take(r, "class")) {
        load_class(r, filename, &classes);
    }
    zo_expect(r, "end");
    zo_eol(r);
    if (r->p != r->end) {
        zo_fail(r, "unexpected data after unit");
    }
    zend_string *key;
    zend_op_array *body;
    ZEND_HASH_MAP_FOREACH_STR_KEY_PTR(&functions, key, body) {
        zend_hash_add_new_ptr(CG(function_table), key, body);
        zend_observer_function_declared_notify(body, key);
    } ZEND_HASH_FOREACH_END();
    install_classes(r, &classes);
    zend_hash_destroy(&classes);
    zend_hash_destroy(&functions);
    if (halt_offset >= 0) {
        key = halt_name(filename);
        zend_constant *existing = zend_hash_find_ptr(EG(zend_constants), key);
        if (existing && Z_LVAL(existing->value) != halt_offset) {
            zo_fail(r, "conflicting compiler halt offset");
        }
        if (!existing) {
            zend_register_long_constant(ZSTR_VAL(key), ZSTR_LEN(key), halt_offset, 0, 0);
        }
        zend_string_release(key);
    }
    zo_validate_unit_calls(r);
    for (size_t n = 0; n < count; n++) {
        if (!zend_is_auto_global(ZSTR_KNOWN(autoglobals[order[n]]))) {
            zo_fail(r, "autoglobal unavailable in this PHP runtime");
        }
    }
    r->filename = NULL;
    zend_string_release(filename);
    return main;
}
