#include "opcodes.h"
#include "Zend/zend_frameless_function.h"
#include "Zend/zend_partial.h"
#include "Zend/zend_vm.h"

typedef enum {
    ZO_EMPTY, ZO_NUMBER, ZO_TARGET, ZO_LITERAL, ZO_CV, ZO_TMP, ZO_VAR
} zo_operand_kind;

typedef struct {
    zo_operand_kind kind;
    uint32_t value;
} zo_operand;

static bool dispatch_table(uint8_t opcode)
{
    return opcode == ZEND_MATCH || opcode == ZEND_SWITCH_LONG || opcode == ZEND_SWITCH_STRING;
}

static bool argument_send(uint8_t opcode)
{
    switch (opcode) {
        case ZEND_SEND_VAL:
        case ZEND_SEND_VAL_EX:
        case ZEND_SEND_VAR:
        case ZEND_SEND_VAR_EX:
        case ZEND_SEND_REF:
        case ZEND_SEND_VAR_NO_REF:
        case ZEND_SEND_VAR_NO_REF_EX:
        case ZEND_SEND_FUNC_ARG:
        case ZEND_SEND_USER:
        case ZEND_SEND_UNPACK:
        case ZEND_SEND_PLACEHOLDER:
            return true;
        default:
            return false;
    }
}

static bool result_cache(const zend_op *op)
{
    switch (op->opcode) {
        case ZEND_INIT_FCALL:
        case ZEND_INIT_FCALL_BY_NAME:
        case ZEND_INIT_NS_FCALL_BY_NAME:
            return true;
        case ZEND_INIT_STATIC_METHOD_CALL:
            return op->op1_type == IS_CONST || op->op2_type == IS_CONST;
        case ZEND_INIT_METHOD_CALL:
        case ZEND_CHECK_FUNC_ARG:
            return op->op2_type == IS_CONST;
        default:
            return argument_send(op->opcode) && op->op2_type == IS_CONST;
    }
}

static bool dimension_key(uint8_t opcode)
{
    switch (opcode) {
        case ZEND_ASSIGN_DIM:
        case ZEND_ASSIGN_DIM_OP:
        case ZEND_FETCH_DIM_R:
        case ZEND_FETCH_DIM_W:
        case ZEND_FETCH_DIM_RW:
        case ZEND_FETCH_DIM_IS:
        case ZEND_FETCH_DIM_FUNC_ARG:
        case ZEND_FETCH_DIM_UNSET:
        case ZEND_FETCH_LIST_R:
        case ZEND_FETCH_LIST_W:
        case ZEND_ISSET_ISEMPTY_DIM_OBJ:
        case ZEND_UNSET_DIM:
            return true;
        default:
            return false;
    }
}

static bool uses_data_row(uint8_t opcode)
{
    switch (opcode) {
        case ZEND_ASSIGN_DIM:
        case ZEND_ASSIGN_DIM_OP:
        case ZEND_ASSIGN_OBJ:
        case ZEND_ASSIGN_OBJ_OP:
        case ZEND_ASSIGN_OBJ_REF:
        case ZEND_ASSIGN_STATIC_PROP:
        case ZEND_ASSIGN_STATIC_PROP_OP:
        case ZEND_ASSIGN_STATIC_PROP_REF:
        case ZEND_DECLARE_ATTRIBUTED_CONST:
        case ZEND_FRAMELESS_ICALL_3:
            return true;
        default:
            return false;
    }
}

static zo_operand_kind unused_kind(const zend_op *op, bool second)
{
    uint32_t flags = zend_get_opcode_flags(op->opcode);
    uint32_t role = (second ? ZEND_VM_OP2_FLAGS(flags) : ZEND_VM_OP1_FLAGS(flags)) & ZEND_VM_OP_MASK;
    if (second && (op->opcode == ZEND_RECV || op->opcode == ZEND_SEND_UNPACK ||
                   op->opcode == ZEND_CALLABLE_CONVERT_PARTIAL)) {
        return ZO_NUMBER;
    }
    switch (role) {
        case ZEND_VM_OP_JMP_ADDR:
            return second && op->opcode == ZEND_CATCH && (op->extended_value & ZEND_LAST_CATCH)
                ? ZO_EMPTY : ZO_TARGET;
        case ZEND_VM_OP_NUM:
        case ZEND_VM_OP_CACHE_SLOT:
        case ZEND_VM_OP_CONST_FETCH:
        case ZEND_VM_OP_CLASS_FETCH:
            return ZO_NUMBER;
        case ZEND_VM_OP_TRY_CATCH:
            return ZO_NUMBER;
        case ZEND_VM_OP_LOOP_END:
            return op->extended_value & ZEND_FREE_ON_RETURN ? ZO_NUMBER : ZO_EMPTY;
        default:
            return ZO_EMPTY;
    }
}

static zo_operand native_operand(const zend_op_array *body, const zend_op *op, uint8_t type,
                                 znode_op node, zo_operand_kind unused)
{
    switch (type) {
        case IS_UNUSED:
            if (unused == ZO_TARGET) {
                return (zo_operand){ZO_TARGET, (uint32_t)(OP_JMP_ADDR(op, node) - body->opcodes)};
            }
            return (zo_operand){unused, unused == ZO_EMPTY ? 0 : node.num};
        case IS_CONST:
            return (zo_operand){ZO_LITERAL, (uint32_t)(RT_CONSTANT(op, node) - body->literals)};
        case IS_CV:
            return (zo_operand){ZO_CV, EX_VAR_TO_NUM(node.var)};
        case IS_TMP_VAR:
            return (zo_operand){ZO_TMP, EX_VAR_TO_NUM(node.var)};
        case IS_VAR:
            return (zo_operand){ZO_VAR, EX_VAR_TO_NUM(node.var)};
        default:
            zo_unsupported("operand type %u on %s", type, zend_get_opcode_name(op->opcode));
    }
}

static void print_operand(smart_str *out, zo_operand operand)
{
    switch (operand.kind) {
        case ZO_EMPTY: smart_str_appendc(out, '_'); return;
        case ZO_NUMBER: break;
        case ZO_TARGET: smart_str_appendc(out, '@'); break;
        case ZO_LITERAL: smart_str_appends(out, "lit["); break;
        case ZO_CV: smart_str_appends(out, "CV"); break;
        case ZO_TMP: smart_str_appendc(out, 'T'); break;
        case ZO_VAR: smart_str_appendc(out, 'V'); break;
    }
    smart_str_append_unsigned(out, operand.value);
    if (operand.kind == ZO_LITERAL) smart_str_appendc(out, ']');
}

uint32_t zo_absolute_target(const zend_op_array *body, uint32_t index, zend_long offset)
{
    if (offset < INT32_MIN || offset > UINT32_MAX) {
        zo_unsupported("jump offset outside the engine's range in instruction %u", index);
    }
    int32_t delta = (int32_t)offset;
    int64_t target = (int64_t)index + delta / (int32_t)sizeof(zend_op);
    if (delta % (int32_t)sizeof(zend_op) || target < 0 || target >= body->last) {
        zo_unsupported("invalid native jump target in instruction %u", index);
    }
    return (uint32_t)target;
}

void zo_print_literal(zo_writer *writer, const zend_op_array *body, uint32_t index)
{
    const zval *literal = &body->literals[index];
    if (Z_TYPE_P(literal) == IS_ARRAY) {
        for (uint32_t i = 0; i < body->last; ++i) {
            const zend_op *op = &body->opcodes[i];
            if (!dispatch_table(op->opcode) || RT_CONSTANT(op, op->op2) != literal) continue;
            zo_put_table(writer, Z_ARR_P(literal), body, i);
            return;
        }
    }
    zo_put_value(writer, literal);
}

void zo_print_instructions(smart_str *out, const zend_op_array *body)
{
    for (uint32_t i = 0; i < body->last; ++i) {
        const zend_op *op = &body->opcodes[i];
        const char *name = zend_get_opcode_name(op->opcode);
        if (!name) zo_unsupported("unknown opcode %u", op->opcode);
        smart_str_append_printf(out, "    %u: ", i);
        uint8_t smart = op->result_type & (IS_SMART_BRANCH_JMPZ | IS_SMART_BRANCH_JMPNZ);
        uint8_t result_type = op->result_type & ~smart;
        if (result_type != IS_UNUSED) {
            print_operand(out, native_operand(body, op, result_type, op->result, ZO_EMPTY));
            smart_str_appends(out, " = ");
        }
        smart_str_appends(out, name + sizeof("ZEND_") - 1);
        zo_operand a = native_operand(body, op, op->op1_type, op->op1, unused_kind(op, false));
        zo_operand b = native_operand(body, op, op->op2_type, op->op2, unused_kind(op, true));
        if (a.kind != ZO_EMPTY || b.kind != ZO_EMPTY) {
            smart_str_appendc(out, ' ');
            print_operand(out, a);
            if (b.kind != ZO_EMPTY) {
                smart_str_appends(out, ", ");
                print_operand(out, b);
            }
        }
        smart_str_append_printf(out, " | line=%u", op->lineno);
        uint32_t ext = op->extended_value;
        if (op->opcode == ZEND_CALLABLE_CONVERT_PARTIAL) ext &= ~ZEND_PARTIAL_CACHEABLE_IN_SHM;
        if (ZEND_OP_IS_FRAMELESS_ICALL(op->opcode)) {
            smart_str_appends(out, " builtin=");
            zo_put_string(out, ZEND_FLF_FUNC(op)->common.function_name);
        } else if ((zend_get_opcode_flags(op->opcode) & ZEND_VM_EXT_MASK) == ZEND_VM_EXT_JMP_ADDR) {
            smart_str_append_printf(out, " ext=@%u", zo_absolute_target(body, i, op->extended_value));
        } else if (ext) {
            smart_str_append_printf(out, " ext=%u", ext);
        }
        if (result_cache(op)) smart_str_append_printf(out, " cache=%u", op->result.num);
        if (op->opcode == ZEND_RECV_INIT && Z_TYPE_P(RT_CONSTANT(op, op->op2)) == IS_CONSTANT_AST) {
            smart_str_append_printf(out, " default-cache=%u", Z_CACHE_SLOT_P(RT_CONSTANT(op, op->op2)));
        }
        if (op->opcode == ZEND_CALLABLE_CONVERT_PARTIAL && op->op2_type == IS_CONST) {
            uint32_t mask = Z_EXTRA_P(RT_CONSTANT(op, op->op2));
            if (mask) smart_str_append_printf(out, " bound-mask=%u", mask);
        }
        if (smart) smart_str_appends(out, smart == IS_SMART_BRANCH_JMPZ ? " smart=JMPZ" : " smart=JMPNZ");
        if (dimension_key(op->opcode) && op->op2_type == IS_CONST &&
            Z_EXTRA_P(RT_CONSTANT(op, op->op2)) == ZEND_EXTRA_VALUE) {
            smart_str_appends(out, " numeric-string-key");
        }
        smart_str_appendc(out, '\n');
    }
}

static void skip_horizontal(zo_reader *r)
{
    while (r->p != r->end && (*r->p == ' ' || *r->p == '\t')) ++r->p;
}

static bool slot_prefix(zo_reader *r, const char *prefix)
{
    size_t n = strlen(prefix);
    if ((size_t)(r->end - r->p) <= n || memcmp(r->p, prefix, n) != 0 ||
        r->p[n] < '0' || r->p[n] > '9') return false;
    r->p += n;
    return true;
}

static zo_operand read_operand(zo_reader *r)
{
    skip_horizontal(r);
    zo_operand_kind kind;
    if (zo_take(r, "_")) return (zo_operand){ZO_EMPTY, 0};
    if (zo_take(r, "@")) kind = ZO_TARGET;
    else if (zo_take(r, "lit[")) kind = ZO_LITERAL;
    else if (slot_prefix(r, "CV")) kind = ZO_CV;
    else if (slot_prefix(r, "T")) kind = ZO_TMP;
    else if (slot_prefix(r, "V")) kind = ZO_VAR;
    else if (r->p < r->end && *r->p >= '0' && *r->p <= '9') kind = ZO_NUMBER;
    else return (zo_operand){ZO_EMPTY, 0};
    uint32_t value = zo_u32(r);
    if (kind == ZO_LITERAL) zo_expect(r, "]");
    return (zo_operand){kind, value};
}

static uint8_t read_opcode(zo_reader *r)
{
    skip_horizontal(r);
    const char *name = r->p;
    while (r->p < r->end && ((*r->p >= 'A' && *r->p <= 'Z') ||
           (*r->p >= '0' && *r->p <= '9') || *r->p == '_')) ++r->p;
    size_t len = (size_t)(r->p - name);
    for (unsigned id = 0; id <= ZEND_VM_LAST_OPCODE; ++id) {
        const char *entry = zend_get_opcode_name(id);
        if (entry && strlen(entry + 5) == len && !memcmp(name, entry + 5, len)) return (uint8_t)id;
    }
    zo_fail(r, "unknown opcode '%.*s'", (int)MIN(len, 80), name);
}

static uint32_t find_builtin(zo_reader *r, uint8_t opcode)
{
    zend_string *name = zo_string(r);
    zend_function *function = zend_hash_find_ptr(EG(function_table), name);
    if (function && function->type == ZEND_INTERNAL_FUNCTION) {
        const zend_frameless_function_info *info = function->internal_function.frameless_function_infos;
        for (; info && info->handler; ++info) {
            if (info->num_args != (uint32_t)ZEND_FLF_NUM_ARGS(opcode)) continue;
            for (uint32_t i = 0; zend_flf_handlers && zend_flf_handlers[i]; ++i) {
                if (zend_flf_handlers[i] == info->handler && zend_flf_functions[i] == function) {
                    zend_string_release(name);
                    return i;
                }
            }
        }
    }
    zo_fail(r, "no frameless builtin '%s' of arity %d", ZSTR_VAL(name), ZEND_FLF_NUM_ARGS(opcode));
}

static void bind_type_assert(zo_reader *r, zend_op *op)
{
    if (op->op1_type != IS_CONST || Z_TYPE_P(RT_CONSTANT(op, op->op1)) != IS_STRING) {
        zo_fail(r, "TYPE_ASSERT requires a builtin name");
    }
    zval *name = RT_CONSTANT(op, op->op1);
    zval *entry = zend_hash_find(EG(function_table), Z_STR_P(name));
    uint32_t arg = op->extended_value >> 16;
    if (!entry || Z_FUNC_P(entry)->type != ZEND_INTERNAL_FUNCTION || !Z_FUNC_P(entry)->common.arg_info ||
        !arg || arg > Z_FUNC_P(entry)->common.num_args) zo_fail(r, "invalid builtin parameter on TYPE_ASSERT");
    Bucket *bucket = ZEND_CONTAINER_OF(entry, Bucket, val);
    Z_EXTRA_P(name) = (uint32_t)(bucket - EG(function_table)->arData);
}

static uint32_t relative_target(zo_reader *r, const zend_op_array *body, uint32_t i, uint32_t target)
{
    if (target >= body->last) zo_fail(r, "instruction %u references missing target @%u", i, target);
    int64_t delta = ((int64_t)target - i) * (int64_t)sizeof(zend_op);
    if (delta < INT32_MIN || delta > INT32_MAX) zo_fail(r, "jump displacement exceeds the engine limit");
    return (uint32_t)(int32_t)delta;
}

static void set_operand(zo_reader *r, const zend_op_array *body, zend_op *op, zo_operand arg,
                        zo_operand_kind unused, uint8_t *type, znode_op *node)
{
    uint32_t value = arg.value;
    switch (arg.kind) {
        case ZO_EMPTY:
        case ZO_TARGET:
        case ZO_NUMBER:
            if (arg.kind != unused) zo_fail(r, "wrong immediate or jump operand for %s", zend_get_opcode_name(op->opcode));
            if (arg.kind == ZO_TARGET) {
                relative_target(r, body, (uint32_t)(op - body->opcodes), value);
                ZEND_SET_OP_JMP_ADDR(op, *node, &body->opcodes[value]);
            } else {
                node->num = arg.kind == ZO_EMPTY ? UINT32_MAX : value;
            }
            *type = IS_UNUSED;
            return;
        case ZO_LITERAL: {
            if (value >= (uint32_t)body->last_literal) zo_fail(r, "missing literal lit[%u]", value);
#if !ZEND_USE_ABS_CONST_ADDR
            ptrdiff_t offset = (char *)&body->literals[value] - (char *)op;
            if (offset > INT32_MAX || offset < INT32_MIN) zo_fail(r, "literal displacement exceeds the engine limit");
#endif
            node->constant = value;
            ZEND_PASS_TWO_UPDATE_CONSTANT(body, op, *node);
            *type = IS_CONST;
            return;
        }
        case ZO_CV:
        case ZO_TMP:
        case ZO_VAR:
            if (value >= UINT32_MAX / sizeof(zval) - ZEND_CALL_FRAME_SLOT ||
                (arg.kind == ZO_CV ? value >= (uint32_t)body->last_var :
                 value < (uint32_t)body->last_var || value - (uint32_t)body->last_var >= body->T)) {
                zo_fail(r, "frame slot %u is outside its declared range", value);
            }
            node->var = EX_NUM_TO_VAR(value);
            *type = arg.kind == ZO_CV ? IS_CV : arg.kind == ZO_TMP ? IS_TMP_VAR : IS_VAR;
            return;
    }
}

static void read_instruction(zo_reader *r, zend_op_array *body, uint32_t i)
{
    if (zo_u32(r) != i) zo_fail(r, "expected instruction %u", i);
    zo_expect(r, ":");
    zend_op *op = &body->opcodes[i];
    MAKE_NOP(op);
    zo_operand result = read_operand(r);
    if (result.kind != ZO_EMPTY) zo_expect(r, "=");
    op->opcode = read_opcode(r);
    zo_operand a = read_operand(r);
    zo_operand b = {ZO_EMPTY, 0};
    if (zo_take(r, ",")) b = read_operand(r);
    uint32_t smart = 0, cache = UINT32_MAX, default_cache = UINT32_MAX, bound_mask = 0;
    bool has_cache = false, has_default_cache = false, has_builtin = false, ext_target = false, numeric_key = false;
    op->lineno = 0;
    op->extended_value = 0;
    if (zo_take(r, "|")) {
        unsigned fields = 0;
        for (;;) {
            unsigned bit;
            if (zo_take(r, "line=")) {
                bit = 1;
                op->lineno = zo_u32(r);
            } else if (zo_take(r, "ext=")) {
                bit = 2;
                ext_target = zo_take(r, "@");
                op->extended_value = zo_u32(r);
            } else if (zo_take(r, "cache=")) {
                bit = 4;
                has_cache = true;
                cache = zo_u32(r);
            } else if (zo_take(r, "default-cache=")) {
                bit = 32;
                has_default_cache = true;
                default_cache = zo_u32(r);
            } else if (zo_take(r, "bound-mask=")) {
                bit = 64;
                if (op->opcode != ZEND_CALLABLE_CONVERT_PARTIAL || b.kind != ZO_LITERAL) {
                    zo_fail(r, "bound-mask requires named partial application placeholders");
                }
                bound_mask = zo_u32(r);
            } else if (zo_take(r, "smart=")) {
                bit = 8;
                if (zo_take(r, "JMPNZ")) smart = IS_SMART_BRANCH_JMPNZ;
                else { zo_expect(r, "JMPZ"); smart = IS_SMART_BRANCH_JMPZ; }
            } else if (zo_take(r, "numeric-string-key")) {
                bit = 16;
                numeric_key = true;
            } else if (zo_take(r, "builtin=")) {
                bit = 2;
                if (!ZEND_OP_IS_FRAMELESS_ICALL(op->opcode)) zo_fail(r, "unexpected builtin field");
                has_builtin = true;
                op->extended_value = find_builtin(r, op->opcode);
            } else break;
            if (fields & bit) zo_fail(r, "duplicate instruction field");
            fields |= bit;
        }
    }
    if (has_builtin != ZEND_OP_IS_FRAMELESS_ICALL(op->opcode)) zo_fail(r, "missing builtin name");
    bool branch_ext = (zend_get_opcode_flags(op->opcode) & ZEND_VM_EXT_MASK) == ZEND_VM_EXT_JMP_ADDR;
    if (ext_target != branch_ext) zo_fail(r, "wrong extended jump target for %s", zend_get_opcode_name(op->opcode));
    if (ext_target) op->extended_value = relative_target(r, body, i, op->extended_value);
    if (op->opcode == ZEND_RECV && b.kind == ZO_EMPTY) b = (zo_operand){ZO_NUMBER, MAY_BE_ANY};
    set_operand(r, body, op, a, unused_kind(op, false), &op->op1_type, &op->op1);
    set_operand(r, body, op, b, unused_kind(op, true), &op->op2_type, &op->op2);
    set_operand(r, body, op, result, ZO_EMPTY, &op->result_type, &op->result);
    if (op->opcode == ZEND_TYPE_ASSERT) bind_type_assert(r, op);
    if (op->opcode == ZEND_CALLABLE_CONVERT_PARTIAL) {
        if (op->extended_value & ZEND_PARTIAL_CACHEABLE_IN_SHM) {
            zo_fail(r, "a loaded partial application cannot use shared-memory storage");
        }
        if (op->op2_type == IS_CONST) {
            zval *positions = RT_CONSTANT(op, op->op2);
            if (Z_TYPE_P(positions) != IS_ARRAY) zo_fail(r, "expected a named placeholder table");
            Z_EXTRA_P(positions) = bound_mask;
        }
    }
    if (has_cache != result_cache(op)) zo_fail(r, "wrong cache field for %s", zend_get_opcode_name(op->opcode));
    bool deferred_default = op->opcode == ZEND_RECV_INIT && op->op2_type == IS_CONST &&
        Z_TYPE_P(RT_CONSTANT(op, op->op2)) == IS_CONSTANT_AST;
    if (deferred_default != has_default_cache) zo_fail(r, "wrong default-cache field");
    if (deferred_default) {
        if (default_cache % 8 || (uint64_t)default_cache + sizeof(zval) > (uint32_t)body->cache_size) {
            zo_fail(r, "default-cache exceeds the body cache");
        }
        Z_CACHE_SLOT_P(RT_CONSTANT(op, op->op2)) = default_cache;
    }
    if (has_cache) {
        if (result.kind != ZO_EMPTY) zo_fail(r, "cache and result use the same operand");
        op->result.num = cache;
    } else if (argument_send(op->opcode) && op->op2_type == IS_UNUSED) {
        uint32_t arg = op->op2.num;
        /* Positional sends address their result with ZEND_CALL_VAR, which casts this byte offset
         * to int. SEND_UNPACK encodes the preceding argument count, but doesn't use the offset. */
        if (op->opcode != ZEND_SEND_UNPACK
                && (arg == 0 || (uint64_t)arg - 1 + ZEND_CALL_FRAME_SLOT > INT_MAX / sizeof(zval))) {
            zo_fail(r, "invalid argument position %u", arg);
        }
        op->result.var = EX_NUM_TO_VAR(arg - 1);
    }
    if (smart && result.kind != ZO_TMP) zo_fail(r, "smart branch requires a temporary result");
    op->result_type |= smart;
    if (dimension_key(op->opcode) && op->opcode != ZEND_UNSET_DIM
            && b.kind == ZO_LITERAL && Z_ISUNDEF(body->literals[b.value])) {
        zo_fail(r, "undef dimension key must be a compiled variable");
    }
    if ((op->opcode == ZEND_FETCH_DIM_R || op->opcode == ZEND_FETCH_LIST_R || op->opcode == ZEND_FETCH_DIM_FUNC_ARG)
            && a.kind == ZO_LITERAL && Z_ISUNDEF(body->literals[a.value])) {
        zo_fail(r, "undef dimension source must be a compiled variable");
    }
    if (numeric_key) {
        if (!dimension_key(op->opcode) || b.kind != ZO_LITERAL || b.value + 1 >= (uint32_t)body->last_literal ||
            Z_TYPE(body->literals[b.value]) != IS_LONG || Z_TYPE(body->literals[b.value + 1]) != IS_STRING) {
            zo_fail(r, "numeric-string-key requires adjacent integer and string literals");
        }
        Z_EXTRA(body->literals[b.value]) = ZEND_EXTRA_VALUE;
    }
    zo_eol(r);
}

static void validate_target(zo_reader *r, const zend_op_array *body, uint32_t executable,
        uint32_t i, uint32_t target)
{
    if (target >= executable || body->opcodes[target].opcode == ZEND_OP_DATA) {
        zo_fail(r, "instruction %u references invalid jump target @%u", i, target);
    }
}

static void fix_targets(zo_reader *r, zend_op_array *body, uint32_t first_line, uint32_t executable)
{
    bool *used = ecalloc(MAX(1, body->last_literal), sizeof(bool));
    for (uint32_t i = 0; i < body->last; ++i) {
        r->line = first_line + i;
        zend_op *op = &body->opcodes[i];
        bool data = i > 0 && uses_data_row(body->opcodes[i - 1].opcode);
        if ((op->opcode == ZEND_OP_DATA) != data || (uses_data_row(op->opcode) && i + 1 == body->last)) {
            zo_fail(r, "instruction %u has an unpaired OP_DATA", i);
        }
        if (op->op1_type == IS_UNUSED && unused_kind(op, false) == ZO_TARGET) {
            validate_target(r, body, executable, i, (uint32_t)(OP_JMP_ADDR(op, op->op1) - body->opcodes));
        }
        if (op->op2_type == IS_UNUSED && unused_kind(op, true) == ZO_TARGET) {
            validate_target(r, body, executable, i, (uint32_t)(OP_JMP_ADDR(op, op->op2) - body->opcodes));
        }
        uint32_t smart = op->result_type & (IS_SMART_BRANCH_JMPZ | IS_SMART_BRANCH_JMPNZ);
        if (smart && (!zend_is_smart_branch(op) || i + 1 >= body->last ||
            body->opcodes[i + 1].opcode != (smart == IS_SMART_BRANCH_JMPZ ? ZEND_JMPZ : ZEND_JMPNZ) ||
            body->opcodes[i + 1].op1_type != IS_TMP_VAR || body->opcodes[i + 1].op1.var != op->result.var)) {
            zo_fail(r, "instruction %u has a mismatched smart branch", i);
        }
        if ((zend_get_opcode_flags(op->opcode) & ZEND_VM_EXT_MASK) == ZEND_VM_EXT_JMP_ADDR) {
            validate_target(r, body, executable, i, (uint32_t)ZEND_OFFSET_TO_OPLINE_NUM(body, op, op->extended_value));
        }
        if (!dispatch_table(op->opcode)) continue;
        if (op->op2_type != IS_CONST) zo_fail(r, "instruction %u requires a jump-table literal", i);
        zval *table = RT_CONSTANT(op, op->op2);
        uint32_t literal = (uint32_t)(table - body->literals);
        if (Z_TYPE_P(table) != IS_ARRAY || used[literal]) zo_fail(r, "invalid or shared jump table at instruction %u", i);
        if (GC_REFCOUNT(Z_ARR_P(table)) != 1) zo_fail(r, "expected a plain jump table at instruction %u", i);
        used[literal] = true;
        zval *target;
        zend_string *key;
        ZEND_HASH_FOREACH_STR_KEY_VAL(Z_ARR_P(table), key, target) {
            if ((op->opcode == ZEND_SWITCH_LONG && key) || (op->opcode == ZEND_SWITCH_STRING && !key) ||
                Z_TYPE_P(target) != IS_LONG || Z_LVAL_P(target) < 0 || Z_LVAL_P(target) > UINT32_MAX) {
                zo_fail(r, "invalid jump table entry at instruction %u", i);
            }
            uint32_t dest = (uint32_t)Z_LVAL_P(target);
            validate_target(r, body, executable, i, dest);
            ZVAL_LONG(target, (int32_t)relative_target(r, body, i, dest));
        } ZEND_HASH_FOREACH_END();
    }
    efree(used);
}

void zo_read_instructions(zo_reader *r, zend_op_array *body, const bool *immutable_literals)
{
    if (!body->last) zo_fail(r, "a body requires at least one instruction");
    uint32_t first_line = r->line;
    for (uint32_t i = 0; i < body->last; ++i) read_instruction(r, body, i);
    uint32_t next_line = r->line;
    /* Nested switches can leave several unreachable cleanup instructions after a return. */
    uint32_t executable = body->last;
    while (executable && body->opcodes[executable - 1].opcode == ZEND_FREE &&
            body->opcodes[executable - 1].extended_value == ZEND_FREE_SWITCH) {
        --executable;
    }
    fix_targets(r, body, first_line, executable);
    /* Pool jump tables now contain the byte offsets used by Zend. */
    for (uint32_t i = 0; i < body->last_literal; ++i) {
        if (immutable_literals[i]) zo_freeze_literal_array(&body->literals[i]);
    }
    r->line = next_line;
    zo_validate_code(r, body, executable);
    for (uint32_t i = 0; i < body->last; ++i) ZEND_VM_SET_OPCODE_HANDLER(&body->opcodes[i]);
}
