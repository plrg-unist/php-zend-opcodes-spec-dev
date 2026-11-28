#include "opcodes.h"
#include "Zend/zend_arena.h"
#include "Zend/zend_extensions.h"

#define ZO_TYPE_MASK (MAY_BE_ANY | MAY_BE_STATIC | MAY_BE_CALLABLE | MAY_BE_VOID | MAY_BE_NEVER \
    | _ZEND_TYPE_ITERABLE_BIT | (3u << _ZEND_SEND_MODE_SHIFT) | _ZEND_IS_VARIADIC_BIT \
    | _ZEND_IS_PROMOTED_BIT | _ZEND_IS_TENTATIVE_BIT)

#define ZO_FUNCTION_FLAGS (ZEND_ACC_PPP_MASK | ZEND_ACC_CHANGED | ZEND_ACC_STATIC | ZEND_ACC_FINAL \
    | ZEND_ACC_ABSTRACT | ZEND_ACC_HAS_TYPE_HINTS | ZEND_ACC_TOP_LEVEL | ZEND_ACC_DEPRECATED \
    | ZEND_ACC_OVERRIDE | ZEND_ACC_RETURN_REFERENCE | ZEND_ACC_HAS_RETURN_TYPE | ZEND_ACC_VARIADIC \
    | ZEND_ACC_HAS_FINALLY_BLOCK | ZEND_ACC_EARLY_BINDING | ZEND_ACC_USES_THIS | ZEND_ACC_CTOR \
    | ZEND_ACC_CLOSURE | ZEND_ACC_GENERATOR | ZEND_ACC_STRICT_TYPES | ZEND_ACC_NODISCARD)
#define ZO_STORAGE_FLAGS (ZEND_ACC_DONE_PASS_TWO | ZEND_ACC_IMMUTABLE | ZEND_ACC_HEAP_RT_CACHE)
#define ZO_MAIN_FLAGS (ZEND_ACC_STRICT_TYPES | ZEND_ACC_HAS_FINALLY_BLOCK \
    | ZEND_ACC_EARLY_BINDING | ZEND_ACC_USES_THIS)

static uint32_t read_count(zo_reader *r)
{
    uint32_t count = zo_u32(r);
    if (count > (size_t)(r->end - r->p)) {
        zo_fail(r, "count exceeds remaining listing length");
    }
    return count;
}

void zo_put_type(smart_str *out, zend_type type)
{
    uint32_t mask = ZEND_TYPE_FULL_MASK(type);
    uint32_t payload = mask & ~(_ZEND_TYPE_KIND_MASK | _ZEND_TYPE_ARENA_BIT
        | _ZEND_TYPE_UNION_BIT | _ZEND_TYPE_INTERSECTION_BIT);
    if ((payload & ~ZO_TYPE_MASK) || ZEND_TYPE_HAS_LITERAL_NAME(type)) {
        zo_unsupported("unsupported user type mask %u", mask);
    }
    smart_str_append_printf(out, "type(%u", payload);
    if (ZEND_TYPE_HAS_NAME(type)) {
        smart_str_appends(out, ", name ");
        zo_put_string(out, ZEND_TYPE_NAME(type));
    } else if (ZEND_TYPE_HAS_LIST(type)) {
        zend_type_list *list = ZEND_TYPE_LIST(type);
        smart_str_append_printf(out, ", %s %u [",
            ZEND_TYPE_IS_INTERSECTION(type) ? "intersection" : "union", list->num_types);
        for (uint32_t i = 0; i < list->num_types; i++) {
            if (i) smart_str_appends(out, ", ");
            zo_put_type(out, list->types[i]);
        }
        smart_str_appendc(out, ']');
    }
    smart_str_appendc(out, ')');
}

static zend_type read_type(zo_reader *r, unsigned int depth)
{
    zo_expect(r, "type(");
    uint32_t mask = zo_u32(r);
    if (mask & ~ZO_TYPE_MASK) {
        zo_fail(r, "unknown type flags %u", mask);
    }
    zend_type type = ZEND_TYPE_INIT_MASK(mask);
    if (zo_take(r, ",")) {
        if (zo_take(r, "name")) {
            zend_string *name = zend_new_interned_string(zo_string(r));
            if (ZSTR_LEN(name) == 0) zo_fail(r, "empty class type name");
            zend_alloc_ce_cache(name);
            ZEND_TYPE_SET_PTR_AND_KIND(type, name, _ZEND_TYPE_NAME_BIT);
        } else {
            bool intersection = zo_take(r, "intersection");
            if (!intersection) zo_expect(r, "union");
            if (depth >= 2) zo_fail(r, "invalid nested type list");
            uint32_t count = read_count(r);
            if (!count) zo_fail(r, "empty type list");
            zend_type_list *list = safe_emalloc(count, sizeof(zend_type), offsetof(zend_type_list, types));
            list->num_types = count;
            zo_expect(r, "[");
            for (uint32_t i = 0; i < count; i++) {
                if (i) zo_expect(r, ",");
                list->types[i] = read_type(r, depth + 1);
                if (!ZEND_TYPE_HAS_NAME(list->types[i])
                        && (intersection || !ZEND_TYPE_IS_INTERSECTION(list->types[i]))) {
                    zo_fail(r, "invalid type list member");
                }
            }
            zo_expect(r, "]");
            ZEND_TYPE_SET_LIST(type, list);
            type.type_mask |= intersection ? _ZEND_TYPE_INTERSECTION_BIT : _ZEND_TYPE_UNION_BIT;
        }
    }
    zo_expect(r, ")");
    return type;
}

zend_type zo_get_type(zo_reader *r)
{
    return read_type(r, 0);
}

static void put_text_field(smart_str *out, const char *name, const zend_string *value)
{
    if (value) {
        smart_str_appends(out, name);
        zo_put_string(out, value);
        smart_str_appendc(out, '\n');
    }
}

static uint32_t argument_count(const zend_op_array *body)
{
    return body->num_args + !!(body->fn_flags & ZEND_ACC_VARIADIC);
}

static void print_arguments(smart_str *out, const zend_op_array *body)
{
    uint32_t count = argument_count(body);
    if (!count && !(body->fn_flags & ZEND_ACC_HAS_RETURN_TYPE)) return;
    smart_str_append_printf(out, "  args %u required %u\n", body->num_args, body->required_num_args);
    if (body->fn_flags & ZEND_ACC_HAS_RETURN_TYPE) {
        smart_str_appends(out, "  returns ");
        zo_put_type(out, body->arg_info[-1].type);
        smart_str_appendc(out, '\n');
    }
    for (uint32_t i = 0; i < count; i++) {
        const zend_arg_info *arg = &body->arg_info[i];
        smart_str_appends(out, "  arg ");
        zo_put_string(out, arg->name);
        smart_str_appendc(out, ' ');
        zo_put_type(out, arg->type);
        if (arg->doc_comment) {
            smart_str_appends(out, " doc ");
            zo_put_string(out, arg->doc_comment);
        }
        smart_str_appendc(out, '\n');
    }
}

static void read_arguments(zo_reader *r, zend_op_array *body)
{
    bool returns = body->fn_flags & ZEND_ACC_HAS_RETURN_TYPE;
    bool variadic = body->fn_flags & ZEND_ACC_VARIADIC;
    if (!zo_take(r, "args")) {
        if (returns || variadic || (body->fn_flags & ZEND_ACC_HAS_TYPE_HINTS)) {
            zo_fail(r, "missing argument information");
        }
        return;
    }
    body->num_args = read_count(r);
    zo_expect(r, "required");
    body->required_num_args = zo_u32(r);
    zo_eol(r);
    if (body->required_num_args > body->num_args
            || body->num_args > UINT32_MAX - 2
            || argument_count(body) > (uint32_t)body->last_var) {
        zo_fail(r, "inconsistent function argument counts");
    }
    uint32_t count = argument_count(body);
    if (returns || count) {
        zend_arg_info *args = ecalloc(count + returns, sizeof(zend_arg_info));
        body->arg_info = args + returns;
    }
    if (returns) {
        zo_expect(r, "returns");
        body->arg_info[-1].type = zo_get_type(r);
        if (!ZEND_TYPE_IS_SET(body->arg_info[-1].type)) zo_fail(r, "missing return type");
        zo_eol(r);
    }
    bool typed_args = false;
    for (uint32_t i = 0; i < count; i++) {
        zend_arg_info *arg = &body->arg_info[i];
        zo_expect(r, "arg");
        arg->name = zo_string(r);
        arg->type = zo_get_type(r);
        if (!zend_string_equals(arg->name, body->vars[i])
                || (bool)ZEND_ARG_IS_VARIADIC(arg) != (variadic && i == body->num_args)
                || ZEND_ARG_SEND_MODE(arg) > ZEND_SEND_PREFER_REF) {
            zo_fail(r, "inconsistent argument %u", i + 1);
        }
        if (zo_take(r, "doc")) arg->doc_comment = zo_string(r);
        typed_args |= ZEND_TYPE_IS_SET(arg->type);
        zo_eol(r);
    }
    if (typed_args != !!(body->fn_flags & ZEND_ACC_HAS_TYPE_HINTS)) {
        zo_fail(r, "type hints disagree with function flags");
    }
    zend_set_function_arg_flags((zend_function *)body);
}

static void print_ranges(smart_str *out, const zend_op_array *body)
{
    if (body->last_try_catch) {
        smart_str_append_printf(out, "  tries %u\n", body->last_try_catch);
        for (uint32_t i = 0; i < body->last_try_catch; i++) {
            const zend_try_catch_element *t = &body->try_catch_array[i];
            smart_str_append_printf(out, "  try @%u catch @%u finally @%u end @%u\n",
                t->try_op, t->catch_op, t->finally_op, t->finally_end);
        }
    }
    if (body->last_live_range) {
        smart_str_append_printf(out, "  ranges %u\n", body->last_live_range);
        for (uint32_t i = 0; i < body->last_live_range; i++) {
            const zend_live_range *l = &body->live_range[i];
            smart_str_append_printf(out, "  live T%u kind %u @%u @%u\n",
                EX_VAR_TO_NUM(l->var & ~ZEND_LIVE_MASK), l->var & ZEND_LIVE_MASK, l->start, l->end);
        }
    }
}

static uint32_t read_target(zo_reader *r, const char *field)
{
    zo_expect(r, field);
    zo_expect(r, "@");
    return zo_u32(r);
}

static void read_ranges(zo_reader *r, zend_op_array *body)
{
    if (zo_take(r, "tries")) {
        body->last_try_catch = read_count(r);
        zo_eol(r);
        body->try_catch_array = ecalloc(body->last_try_catch, sizeof(zend_try_catch_element));
        for (uint32_t i = 0; i < body->last_try_catch; i++) {
            zend_try_catch_element *t = &body->try_catch_array[i];
            t->try_op = read_target(r, "try");
            t->catch_op = read_target(r, "catch");
            t->finally_op = read_target(r, "finally");
            t->finally_end = read_target(r, "end");
            if (t->try_op >= body->last || t->catch_op >= body->last
                    || t->finally_op >= body->last || t->finally_end >= body->last
                    || (t->catch_op && t->catch_op < t->try_op)
                    || (t->finally_op && t->finally_op < MAX(t->try_op, t->catch_op))
                    || (!!t->finally_op != !!t->finally_end) || t->finally_end < t->finally_op
                    || (i && t->try_op < body->try_catch_array[i - 1].try_op)) {
                zo_fail(r, "invalid try/catch region %u", i);
            }
            zo_eol(r);
        }
    }
    if (zo_take(r, "ranges")) {
        body->last_live_range = read_count(r);
        zo_eol(r);
        body->live_range = ecalloc(body->last_live_range, sizeof(zend_live_range));
        for (uint32_t i = 0; i < body->last_live_range; i++) {
            zend_live_range *l = &body->live_range[i];
            zo_expect(r, "live");
            zo_expect(r, "T");
            uint32_t slot = zo_u32(r);
            zo_expect(r, "kind");
            uint32_t kind = zo_u32(r);
            zo_expect(r, "@");
            l->start = zo_u32(r);
            zo_expect(r, "@");
            l->end = zo_u32(r);
            if (slot < (uint32_t)body->last_var || slot >= (uint32_t)body->last_var + body->T
                    || kind > ZEND_LIVE_NEW || l->start >= l->end || l->end > body->last
                    || (i && l->start < body->live_range[i - 1].start)) {
                zo_fail(r, "invalid live range %u", i);
            }
            l->var = EX_NUM_TO_VAR(slot) | kind;
            zo_eol(r);
        }
    }
}

static bool check_attribute_literals(const zend_op_array *body, bool *has_attributes)
{
    uint32_t tables = 0, owners = 0;
    for (uint32_t i = 0; i < body->last_literal; i++) {
        if (Z_TYPE(body->literals[i]) == IS_PTR) tables++;
    }
    *has_attributes = tables != 0;
    if (!tables) return true;
    if (body->function_name) return false;

    bool valid = true;
    bool *owned = ecalloc(body->last_literal, sizeof(bool));
    for (uint32_t i = 0; valid && i < body->last; i++) {
        const zend_op *op = &body->opcodes[i];
        if (op->op2_type == IS_CONST && Z_TYPE_P(RT_CONSTANT(op, op->op2)) == IS_PTR) {
            valid = false;
        }
        if (op->op1_type != IS_CONST) continue;
        const zval *literal = RT_CONSTANT(op, op->op1);
        if (Z_TYPE_P(literal) != IS_PTR) continue;
        uint32_t index = literal - body->literals;
        if (i == 0 || body->opcodes[i - 1].opcode != ZEND_DECLARE_ATTRIBUTED_CONST ||
                op->opcode != ZEND_OP_DATA || !Z_PTR_P(literal) || owned[index]) {
            valid = false;
        } else {
            owned[index] = true;
            owners++;
        }
    }
    efree(owned);
    return valid && tables == owners;
}

void zo_print_body(zo_writer *writer, const zend_op_array *body)
{
    smart_str *out = writer->out;
    if (body->type != ZEND_USER_FUNCTION || !(body->fn_flags & ZEND_ACC_DONE_PASS_TWO)) {
        zo_unsupported("expected a finalized user op array");
    }
    uint32_t flags = body->fn_flags & ~ZO_STORAGE_FLAGS;
    bool has_attributes;
    if (!check_attribute_literals(body, &has_attributes)
            || (has_attributes && !(flags & ZEND_ACC_PTR_OPS))) {
        zo_unsupported("pointer literal without a unique global constant attribute owner");
    }
    if (!body->function_name) flags &= ~ZEND_ACC_PTR_OPS;
    if ((flags & ~ZO_FUNCTION_FLAGS) || (!body->function_name && (flags & ~ZO_MAIN_FLAGS))
            || (body->fn_flags2 & ~ZEND_ACC2_FORBID_DYN_CALLS)) {
        zo_unsupported("unsupported function flags %u/%u", flags, body->fn_flags2);
    }
    smart_str_appends(out, "body ");
    if (body->function_name) zo_put_string(out, body->function_name);
    else smart_str_appendc(out, '-');
    smart_str_appends(out, " {\n");
    if (flags || body->fn_flags2) smart_str_append_printf(out, "  flags %u %u\n", flags, body->fn_flags2);
    if (body->scope) put_text_field(out, "  scope ", body->scope->name);
    smart_str_append_printf(out, "  lines %u %u\n", body->line_start, body->line_end);
    put_text_field(out, "  doc ", body->doc_comment);
    smart_str_append_printf(out, "  frame %u cache %u\n", body->T, body->cache_size);
    if (body->last_var) {
        smart_str_append_printf(out, "  locals %d [", body->last_var);
        for (int i = 0; i < body->last_var; i++) {
            if (i) smart_str_appends(out, ", ");
            zo_put_string(out, body->vars[i]);
        }
        smart_str_appends(out, "]\n");
    }
    print_arguments(out, body);
    zo_put_attributes(writer, body->attributes);
    if (body->static_variables) {
        if (!HT_IS_WITHOUT_HOLES(body->static_variables)) {
            zo_unsupported("static variable table contains gaps");
        }
        smart_str_appends(out, "  statics ");
        zo_put_table(writer, body->static_variables, NULL, 0);
        smart_str_appendc(out, '\n');
    }
    smart_str_append_printf(out, "  size %u literals %u children %u\n",
        body->last, body->last_literal, body->num_dynamic_func_defs);
    for (uint32_t i = 0; i < body->last_literal; i++) {
        smart_str_append_printf(out, "  lit[%u] = ", i);
        if (Z_TYPE(body->literals[i]) == IS_PTR) {
            smart_str_appends(out, "attributes {\n");
            zo_put_attributes(writer, Z_PTR(body->literals[i]));
            smart_str_appendc(out, '}');
        } else {
            zo_print_literal(writer, body, i);
        }
        smart_str_appendc(out, '\n');
    }
    print_ranges(out, body);
    for (uint32_t i = 0; i < body->num_dynamic_func_defs; i++) {
        zo_print_body(writer, body->dynamic_func_defs[i]);
    }
    smart_str_appends(out, "  code\n");
    zo_print_instructions(out, body);
    smart_str_appends(out, "}\n");
}

static void allocate_code(zo_reader *r, zend_op_array *body)
{
    if (body->last > (INT32_MAX - 15u) / sizeof(zend_op)) {
        zo_fail(r, "opcode array exceeds relative address range");
    }
    size_t op_size = ZEND_MM_ALIGNED_SIZE_EX(body->last * sizeof(zend_op), 16);
    if (op_size > INT32_MAX || body->last_literal > (INT32_MAX - op_size) / sizeof(zval)) {
        zo_fail(r, "opcode and literal pool exceed relative address range");
    }
#if ZEND_USE_ABS_CONST_ADDR
    body->opcodes = erealloc(body->opcodes, op_size);
    memset(body->opcodes, 0, op_size);
    if (body->last_literal) body->literals = ecalloc(body->last_literal, sizeof(zval));
#else
    size_t size = op_size + body->last_literal * sizeof(zval);
    body->opcodes = erealloc(body->opcodes, size);
    memset(body->opcodes, 0, size);
    if (body->last_literal) body->literals = (zval *)((char *)body->opcodes + op_size);
#endif
}

void zo_load_body(zo_reader *r, zend_op_array *body, zend_string *filename, zend_class_entry *scope)
{
    zo_expect(r, "body");
    zend_string *name = zo_take(r, "-") ? NULL : zo_string(r);
    if (name && !ZSTR_LEN(name)) zo_fail(r, "empty function name");
    const zend_string *old_context = r->body;
    zend_string *old_source = r->filename;
    zend_class_entry *old_scope = r->scope;
    r->body = name;
    r->filename = filename;
    zo_expect(r, "{");
    zo_eol(r);

    zend_string *old_filename = zend_get_compiled_filename();
    zend_set_compiled_filename(filename);
    init_op_array(body, ZEND_USER_FUNCTION, 1);
    zend_restore_compiled_filename(old_filename);
    body->function_name = name;
    if (zo_take(r, "flags")) {
        body->fn_flags = zo_u32(r);
        body->fn_flags2 = zo_u32(r);
        if ((body->fn_flags & ~ZO_FUNCTION_FLAGS) || (body->fn_flags2 & ~ZEND_ACC2_FORBID_DYN_CALLS)
                || (!name && (body->fn_flags & ~ZO_MAIN_FLAGS))) {
            zo_fail(r, "unsupported function flags");
        }
        zo_eol(r);
    }
    body->fn_flags |= ZEND_ACC_DONE_PASS_TWO;
    if (!name) body->fn_flags |= ZEND_ACC_HEAP_RT_CACHE;
    if (zo_take(r, "scope")) {
        zend_string *scope_name = zo_string(r);
        if (!scope || !zend_string_equals(scope_name, scope->name)) zo_fail(r, "unexpected function scope");
        zend_string_release(scope_name);
        body->scope = scope;
        zo_eol(r);
    }
    r->scope = body->scope ? body->scope : (body->fn_flags & ZEND_ACC_CLOSURE ? scope : NULL);
    zo_expect(r, "lines");
    body->line_start = zo_u32(r);
    body->line_end = zo_u32(r);
    if (body->line_end < body->line_start) zo_fail(r, "invalid source line range");
    zo_eol(r);
    if (zo_take(r, "doc")) {
        body->doc_comment = zo_string(r);
        zo_eol(r);
    }
    zo_expect(r, "frame");
    body->T = zo_u32(r);
    zo_expect(r, "cache");
    uint32_t cache_size = zo_u32(r);
    if ((cache_size % sizeof(void *)) || cache_size < body->cache_size) {
        zo_fail(r, "invalid runtime cache size for this engine");
    }
    body->cache_size = cache_size;
    zo_eol(r);
    if (zo_take(r, "locals")) {
        uint32_t count = read_count(r);
        if (count > INT_MAX) zo_fail(r, "too many locals");
        body->last_var = count;
        if (count) body->vars = ecalloc(count, sizeof(zend_string *));
        zo_expect(r, "[");
        for (uint32_t i = 0; i < count; i++) {
            if (i) zo_expect(r, ",");
            body->vars[i] = zo_string(r);
        }
        zo_expect(r, "]");
        zo_eol(r);
        if (count > 1) {
            HashTable names;
            zend_hash_init(&names, count, NULL, NULL, false);
            for (uint32_t i = 0; i < count; i++) {
                if (!zend_hash_add_empty_element(&names, body->vars[i])) {
                    zend_hash_destroy(&names);
                    zo_fail(r, "duplicate compiled variable name");
                }
            }
            zend_hash_destroy(&names);
        }
    }
    if ((uint64_t)body->last_var + body->T + ZEND_CALL_FRAME_SLOT > UINT32_MAX / sizeof(zval)) {
        zo_fail(r, "frame exceeds Zend stack slot range");
    }
    read_arguments(r, body);
    if (!name && (body->num_args || body->arg_info)) zo_fail(r, "main body cannot have parameters");
    body->attributes = zo_get_attributes(r, argument_count(body));
    if (zo_take(r, "statics")) {
        body->static_variables = zo_get_static_table(r);
        if (!HT_IS_WITHOUT_HOLES(body->static_variables)) zo_fail(r, "static variable table contains gaps");
        zend_string *key;
        ZEND_HASH_FOREACH_STR_KEY(body->static_variables, key) {
            if (!key) zo_fail(r, "invalid static variable name");
        } ZEND_HASH_FOREACH_END();
        zo_eol(r);
    }
    zo_expect(r, "size");
    body->last = read_count(r);
    zo_expect(r, "literals");
    body->last_literal = read_count(r);
    zo_expect(r, "children");
    body->num_dynamic_func_defs = read_count(r);
    if (!body->last) zo_fail(r, "body has no instructions");
    zo_eol(r);
    allocate_code(r, body);
    bool *immutable_literals = ecalloc(MAX(1, body->last_literal), sizeof(bool));
    for (uint32_t i = 0; i < body->last_literal; i++) {
        zo_expect(r, "lit[");
        if (zo_u32(r) != i) zo_fail(r, "expected literal %u", i);
        zo_expect(r, "]");
        zo_expect(r, "=");
        if (zo_take(r, "attributes")) {
            zo_expect(r, "{");
            zo_eol(r);
            HashTable *attributes = zo_get_attributes(r, 0);
            if (!attributes) zo_fail(r, "empty global constant attribute list");
            zo_expect(r, "}");
            ZVAL_PTR(&body->literals[i], attributes);
        } else {
            immutable_literals[i] = zo_get_pool_value(r, &body->literals[i]);
        }
        zo_eol(r);
    }
    read_ranges(r, body);
    if (body->num_dynamic_func_defs) {
        body->dynamic_func_defs = ecalloc(body->num_dynamic_func_defs, sizeof(zend_op_array *));
        for (uint32_t i = 0; i < body->num_dynamic_func_defs; i++) {
            zend_op_array *child = zend_arena_alloc(&CG(arena), sizeof(zend_op_array));
            zo_load_body(r, child, filename, r->scope);
            if (!child->function_name) zo_fail(r, "nested body requires function name");
            body->dynamic_func_defs[i] = child;
        }
    }
    zo_expect(r, "code");
    zo_eol(r);
    zo_read_instructions(r, body, immutable_literals);
    efree(immutable_literals);
    bool has_attributes;
    if (!check_attribute_literals(body, &has_attributes)) {
        zo_fail(r, "attribute literal must belong to exactly one global constant declaration");
    }
    if (has_attributes) body->fn_flags |= ZEND_ACC_PTR_OPS;
    zo_expect(r, "}");
    zo_eol(r);
    r->body = old_context;
    r->filename = old_source;
    r->scope = old_scope;
}
