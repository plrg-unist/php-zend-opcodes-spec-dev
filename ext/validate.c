#include "opcodes.h"
#include "Zend/zend_partial.h"
#include "Zend/zend_vm_opcodes.h"

/* IS_UNUSED is zero in an opline, so it needs a separate bit in these sets. */
enum operand_set {
    unused = 16,
    literal = IS_CONST,
    cv = IS_CV,
    tmp = IS_TMP_VAR,
    var = IS_VAR,
    temp = tmp | var,
    slot = temp | cv,
    value = slot | literal,
    rvalue = literal | tmp | cv,
    writable = cv | var,
    optional = value | unused,
    optional_slot = slot | unused,
    class_ref = literal | unused | var
};

typedef struct {
    uint8_t op1, op2, result;
} operand_shape;

struct zo_pending_body {
    const zend_op_array *body;
    struct zo_pending_body *next;
};

#define SHAPE(name, a, b, c) [ZEND_##name] = { a, b, c }
static const operand_shape shapes[ZEND_VM_LAST_OPCODE + 1] = {
    SHAPE(NOP, unused, unused, unused),
    SHAPE(ADD, value, value, slot),
    SHAPE(SUB, value, value, slot),
    SHAPE(MUL, value, value, slot),
    SHAPE(DIV, rvalue, rvalue, slot),
    SHAPE(MOD, value, value, slot),
    SHAPE(SL, value, value, slot),
    SHAPE(SR, value, value, slot),
    SHAPE(CONCAT, rvalue, rvalue, slot),
    SHAPE(FAST_CONCAT, rvalue, rvalue, slot),
    SHAPE(BW_OR, value, value, slot),
    SHAPE(BW_AND, value, value, slot),
    SHAPE(BW_XOR, value, value, slot),
    SHAPE(POW, rvalue, rvalue, slot),
    SHAPE(BW_NOT, value, unused, slot),
    SHAPE(BOOL_NOT, rvalue, unused, slot),
    SHAPE(BOOL_XOR, rvalue, rvalue, slot),
    SHAPE(IS_IDENTICAL, rvalue, rvalue, slot),
    SHAPE(IS_NOT_IDENTICAL, rvalue, rvalue, slot),
    SHAPE(IS_EQUAL, rvalue, rvalue, slot),
    SHAPE(IS_NOT_EQUAL, rvalue, rvalue, slot),
    SHAPE(IS_SMALLER, value, value, slot),
    SHAPE(IS_SMALLER_OR_EQUAL, value, value, slot),
    SHAPE(SPACESHIP, rvalue, rvalue, slot),
    SHAPE(ASSIGN, writable, rvalue, optional_slot),
    SHAPE(ASSIGN_DIM, writable, optional, optional_slot),
    SHAPE(ASSIGN_OBJ, writable | unused, value, optional_slot),
    SHAPE(ASSIGN_STATIC_PROP, value, class_ref, optional_slot),
    SHAPE(ASSIGN_OP, writable, rvalue, optional_slot),
    SHAPE(ASSIGN_DIM_OP, writable, optional, optional_slot),
    SHAPE(ASSIGN_OBJ_OP, writable | unused, value, optional_slot),
    SHAPE(ASSIGN_STATIC_PROP_OP, value, class_ref, optional_slot),
    SHAPE(ASSIGN_REF, writable, writable, optional_slot),
    SHAPE(ASSIGN_OBJ_REF, writable | unused, value, optional_slot),
    SHAPE(ASSIGN_STATIC_PROP_REF, value, class_ref, optional_slot),
    SHAPE(QM_ASSIGN, value, unused, slot),
    SHAPE(PRE_INC, writable, unused, optional_slot),
    SHAPE(PRE_DEC, writable, unused, optional_slot),
    SHAPE(POST_INC, writable, unused, slot),
    SHAPE(POST_DEC, writable, unused, slot),
    SHAPE(PRE_INC_STATIC_PROP, value, class_ref, optional_slot),
    SHAPE(PRE_DEC_STATIC_PROP, value, class_ref, optional_slot),
    SHAPE(POST_INC_STATIC_PROP, value, class_ref, slot),
    SHAPE(POST_DEC_STATIC_PROP, value, class_ref, slot),
    SHAPE(JMP, unused, unused, unused),
    SHAPE(JMPZ, rvalue, unused, unused),
    SHAPE(JMPNZ, rvalue, unused, unused),
    SHAPE(JMPZ_EX, rvalue, unused, slot),
    SHAPE(JMPNZ_EX, rvalue, unused, slot),
    SHAPE(CASE, tmp, rvalue, slot),
    SHAPE(CASE_STRICT, tmp, rvalue, slot),
    SHAPE(CHECK_VAR, cv, unused, unused),
    SHAPE(CAST, rvalue, unused, slot),
    SHAPE(BOOL, rvalue, unused, slot),
    SHAPE(ROPE_INIT, unused, rvalue, tmp),
    SHAPE(ROPE_ADD, tmp, rvalue, tmp),
    SHAPE(ROPE_END, tmp, rvalue, slot),
    SHAPE(BEGIN_SILENCE, unused, unused, tmp),
    SHAPE(END_SILENCE, tmp, unused, unused),
    SHAPE(INIT_FCALL_BY_NAME, unused, literal, unused),
    SHAPE(INIT_FCALL, unused, literal, unused),
    SHAPE(INIT_NS_FCALL_BY_NAME, unused, literal, unused),
    SHAPE(INIT_METHOD_CALL, optional, value, unused),
    SHAPE(INIT_STATIC_METHOD_CALL, class_ref, optional, unused),
    SHAPE(INIT_DYNAMIC_CALL, unused, value, unused),
    SHAPE(INIT_USER_CALL, literal, value, unused),
    SHAPE(DO_FCALL, unused, unused, optional_slot),
    SHAPE(DO_ICALL, unused, unused, optional_slot),
    SHAPE(DO_UCALL, unused, unused, optional_slot),
    SHAPE(DO_FCALL_BY_NAME, unused, unused, optional_slot),
    SHAPE(RETURN, rvalue, unused, unused),
    SHAPE(RETURN_BY_REF, value, unused, unused),
    SHAPE(RECV, unused, unused, cv),
    SHAPE(RECV_INIT, unused, literal, cv),
    SHAPE(RECV_VARIADIC, unused, unused, cv),
    SHAPE(SEND_VAL, literal | temp, unused | literal, unused),
    SHAPE(SEND_VAL_EX, literal | temp, unused | literal, unused),
    SHAPE(SEND_VAR, writable, unused | literal, unused),
    SHAPE(SEND_VAR_EX, writable, unused | literal, unused),
    SHAPE(SEND_REF, writable, unused | literal, unused),
    SHAPE(SEND_VAR_NO_REF, var, unused | literal, unused),
    SHAPE(SEND_VAR_NO_REF_EX, var, unused | literal, unused),
    SHAPE(CHECK_FUNC_ARG, unused, unused | literal, unused),
    SHAPE(SEND_FUNC_ARG, var, unused | literal, unused),
    SHAPE(SEND_UNPACK, value, unused, unused),
    SHAPE(SEND_ARRAY, value, optional, unused),
    SHAPE(SEND_USER, value, unused, unused),
    SHAPE(CHECK_UNDEF_ARGS, unused, unused, unused),
    SHAPE(NEW, class_ref, unused, slot),
    SHAPE(FREE, temp, unused, unused),
    SHAPE(FE_FREE, temp, unused, unused),
    SHAPE(INIT_ARRAY, optional, rvalue | unused, slot),
    SHAPE(ADD_ARRAY_ELEMENT, value, rvalue | unused, slot),
    SHAPE(ADD_ARRAY_UNPACK, value, unused, slot),
    SHAPE(INCLUDE_OR_EVAL, value, unused, optional_slot),
    SHAPE(UNSET_VAR, value, unused, unused),
    SHAPE(UNSET_DIM, writable, value, unused),
    SHAPE(UNSET_OBJ, writable | unused, value, unused),
    SHAPE(UNSET_CV, cv, unused, unused),
    SHAPE(UNSET_STATIC_PROP, value, class_ref, unused),
    SHAPE(FE_RESET_R, value, unused, temp),
    SHAPE(FE_RESET_RW, value, unused, temp),
    SHAPE(FE_FETCH_R, temp, slot, optional_slot),
    SHAPE(FE_FETCH_RW, var, slot, optional_slot),
    SHAPE(FETCH_R, value, unused, slot),
    SHAPE(FETCH_W, value, unused, slot),
    SHAPE(FETCH_RW, value, unused, slot),
    SHAPE(FETCH_IS, value, unused, slot),
    SHAPE(FETCH_FUNC_ARG, value, unused, slot),
    SHAPE(FETCH_UNSET, value, unused, slot),
    SHAPE(FETCH_DIM_R, value, rvalue, slot),
    SHAPE(FETCH_DIM_W, writable, optional, slot),
    SHAPE(FETCH_DIM_RW, writable, optional, slot),
    SHAPE(FETCH_DIM_IS, value, rvalue, slot),
    SHAPE(FETCH_DIM_FUNC_ARG, value, optional, slot),
    SHAPE(FETCH_DIM_UNSET, writable, value, slot),
    SHAPE(FETCH_OBJ_R, optional, value, slot),
    SHAPE(FETCH_OBJ_W, writable | unused, value, slot),
    SHAPE(FETCH_OBJ_RW, writable | unused, value, slot),
    SHAPE(FETCH_OBJ_IS, optional, value, slot),
    SHAPE(FETCH_OBJ_FUNC_ARG, optional, value, slot),
    SHAPE(FETCH_OBJ_UNSET, writable | unused, value, slot),
    SHAPE(FETCH_LIST_R, value, value, slot),
    SHAPE(FETCH_LIST_W, var, value, slot),
    SHAPE(FETCH_CONSTANT, unused, literal, slot),
    SHAPE(FETCH_CLASS_CONSTANT, class_ref, value, slot),
    SHAPE(FETCH_CLASS, unused, optional, slot),
    SHAPE(FETCH_CLASS_NAME, optional, unused, slot),
    SHAPE(FETCH_STATIC_PROP_R, value, class_ref, slot),
    SHAPE(FETCH_STATIC_PROP_W, value, class_ref, slot),
    SHAPE(FETCH_STATIC_PROP_RW, value, class_ref, slot),
    SHAPE(FETCH_STATIC_PROP_IS, value, class_ref, slot),
    SHAPE(FETCH_STATIC_PROP_FUNC_ARG, value, class_ref, slot),
    SHAPE(FETCH_STATIC_PROP_UNSET, value, class_ref, slot),
    SHAPE(EXT_STMT, unused, unused, unused),
    SHAPE(EXT_FCALL_BEGIN, unused, unused, unused),
    SHAPE(EXT_FCALL_END, unused, unused, unused),
    SHAPE(EXT_NOP, unused, unused, unused),
    SHAPE(TICKS, unused, unused, unused),
    SHAPE(CATCH, literal, unused, cv | unused),
    SHAPE(THROW, value, unused, unused),
    SHAPE(CLONE, optional, unused, slot),
    SHAPE(ISSET_ISEMPTY_VAR, value, unused, slot),
    SHAPE(ISSET_ISEMPTY_DIM_OBJ, rvalue, rvalue, slot),
    SHAPE(ISSET_ISEMPTY_PROP_OBJ, optional, value, slot),
    SHAPE(ISSET_ISEMPTY_CV, cv, unused, slot),
    SHAPE(ISSET_ISEMPTY_STATIC_PROP, value, class_ref, slot),
    SHAPE(ISSET_ISEMPTY_THIS, unused, unused, slot),
    SHAPE(STRLEN, rvalue, unused, slot),
    SHAPE(DEFINED, literal, unused, slot),
    SHAPE(TYPE_CHECK, rvalue, unused, slot),
    SHAPE(VERIFY_RETURN_TYPE, optional, unused, optional_slot),
    SHAPE(VERIFY_NEVER_TYPE, unused, unused, unused),
    SHAPE(PRE_INC_OBJ, writable | unused, value, optional_slot),
    SHAPE(PRE_DEC_OBJ, writable | unused, value, optional_slot),
    SHAPE(POST_INC_OBJ, writable | unused, value, slot),
    SHAPE(POST_DEC_OBJ, writable | unused, value, slot),
    SHAPE(ECHO, rvalue, unused, unused),
    SHAPE(OP_DATA, value, unused, unused),
    SHAPE(INSTANCEOF, slot, class_ref, slot),
    SHAPE(GENERATOR_CREATE, unused, unused, unused),
    SHAPE(MAKE_REF, writable, unused, optional_slot),
    SHAPE(DECLARE_FUNCTION, literal, unused, unused),
    SHAPE(DECLARE_LAMBDA_FUNCTION, unused, unused, slot),
    SHAPE(DECLARE_CONST, literal, literal, unused),
    SHAPE(DECLARE_CLASS, literal, unused | literal, unused),
    SHAPE(DECLARE_CLASS_DELAYED, literal, literal, unused),
    SHAPE(DECLARE_ANON_CLASS, literal, unused | literal, slot),
    SHAPE(ASSERT_CHECK, unused, unused, optional_slot),
    SHAPE(JMP_SET, rvalue, unused, slot),
    SHAPE(SEPARATE, var, unused, optional_slot),
    SHAPE(DISCARD_EXCEPTION, temp, unused, unused),
    SHAPE(YIELD, optional, optional, optional_slot),
    SHAPE(GENERATOR_RETURN, value, unused, unused),
    SHAPE(FAST_CALL, unused, optional, tmp),
    SHAPE(FAST_RET, tmp, unused, unused),
    SHAPE(YIELD_FROM, value, unused, optional_slot),
    SHAPE(COPY_TMP, temp, unused, slot),
    SHAPE(BIND_GLOBAL, cv, literal, unused),
    SHAPE(COALESCE, rvalue, unused, slot),
    SHAPE(FUNC_NUM_ARGS, unused, unused, slot),
    SHAPE(FUNC_GET_ARGS, unused | literal, unused, slot),
    SHAPE(BIND_LEXICAL, tmp, cv, unused),
    SHAPE(BIND_STATIC, cv, optional, unused),
    SHAPE(BIND_INIT_STATIC_OR_JMP, cv, unused, unused),
    SHAPE(FETCH_THIS, unused, unused, slot),
    SHAPE(SWITCH_LONG, value, literal, unused),
    SHAPE(SWITCH_STRING, value, literal, unused),
    SHAPE(MATCH, value, literal, unused),
    SHAPE(IN_ARRAY, rvalue, literal, slot),
    SHAPE(COUNT, rvalue, unused, slot),
    SHAPE(GET_CLASS, optional, unused, slot),
    SHAPE(GET_CALLED_CLASS, unused, unused, slot),
    SHAPE(GET_TYPE, rvalue, unused, slot),
    SHAPE(ARRAY_KEY_EXISTS, rvalue, rvalue, slot),
    SHAPE(MATCH_ERROR, value, unused, unused),
    SHAPE(JMP_NULL, rvalue, unused, slot),
    SHAPE(FETCH_GLOBALS, unused, unused, slot),
    SHAPE(CALLABLE_CONVERT, unused, unused, slot),
    SHAPE(FRAMELESS_ICALL_0, unused, unused, slot),
    SHAPE(FRAMELESS_ICALL_1, value, unused, slot),
    SHAPE(FRAMELESS_ICALL_2, value, value, slot),
    SHAPE(FRAMELESS_ICALL_3, value, value, slot),
    SHAPE(JMP_FRAMELESS, literal, unused, unused),
    SHAPE(TYPE_ASSERT, literal, value, unused),
    SHAPE(INIT_PARENT_PROPERTY_HOOK_CALL, literal, unused, unused),
    SHAPE(DECLARE_ATTRIBUTED_CONST, literal, literal, unused),
    SHAPE(SEND_PLACEHOLDER, unused, literal | unused, unused),
    SHAPE(CALLABLE_CONVERT_PARTIAL, literal, literal | unused, slot),
};
#undef SHAPE

static uint8_t operand_bit(uint8_t type)
{
    return type == IS_UNUSED ? unused : type;
}

static void invalid(zo_reader *r, const zend_op_array *body, const zend_op *op, const char *reason)
{
    zo_fail(r, "instruction %u (%s): %s", (uint32_t)(op - body->opcodes),
        zend_get_opcode_name(op->opcode), reason);
}

static void name_literals(zo_reader *r, const zend_op_array *body, const zend_op *op,
        znode_op node, unsigned count)
{
    const zval *names = RT_CONSTANT(op, node);
    if (count > (size_t)(body->literals + body->last_literal - names)) {
        invalid(r, body, op, "missing related name literals");
    }
    for (unsigned i = 0; i < count; ++i) {
        if (Z_TYPE(names[i]) != IS_STRING) {
            invalid(r, body, op, "expected a string in name literal group");
        }
    }
}

static void cache(zo_reader *r, const zend_op_array *body, const zend_op *op,
        uint32_t offset, unsigned count)
{
    size_t bytes = count * sizeof(void *);
    if (offset % sizeof(void *) || offset > body->cache_size
            || bytes > body->cache_size - offset) {
        invalid(r, body, op, "cache slot outside the body's runtime cache");
    }
}

static void check_names_and_cache(zo_reader *r, const zend_op_array *b, const zend_op *op)
{
    uint32_t ext = op->extended_value;
    bool one = op->op1_type == IS_CONST, two = op->op2_type == IS_CONST;
    switch (op->opcode) {
        case ZEND_INIT_FCALL:
        case ZEND_INIT_FCALL_BY_NAME:
        case ZEND_INIT_NS_FCALL_BY_NAME:
            name_literals(r, b, op, op->op2, op->opcode == ZEND_INIT_FCALL ? 1 :
                (op->opcode == ZEND_INIT_FCALL_BY_NAME ? 2 : 3));
            cache(r, b, op, op->result.num, 1);
            break;
        case ZEND_INIT_STATIC_METHOD_CALL:
        case ZEND_INIT_METHOD_CALL:
            if (one && op->opcode == ZEND_INIT_STATIC_METHOD_CALL) {
                name_literals(r, b, op, op->op1, 2);
                if (!two) cache(r, b, op, op->result.num, 1);
            }
            if (two) {
                name_literals(r, b, op, op->op2, 2);
                cache(r, b, op, op->result.num, 2);
            }
            break;
        case ZEND_NEW:
            if (one) {
                name_literals(r, b, op, op->op1, 2);
                cache(r, b, op, op->op2.num, 1);
            }
            break;
        case ZEND_CATCH:
            name_literals(r, b, op, op->op1, 2);
            cache(r, b, op, ext & ~ZEND_LAST_CATCH, 1);
            break;
        case ZEND_FETCH_CLASS_CONSTANT:
            if (one) name_literals(r, b, op, op->op1, 2);
            if (one || two || ext) cache(r, b, op, ext, 2);
            break;
        case ZEND_FETCH_CLASS:
        case ZEND_INSTANCEOF:
            if (two) {
                name_literals(r, b, op, op->op2, 2);
                cache(r, b, op, ext, 1);
            }
            break;
        case ZEND_FETCH_CONSTANT:
            name_literals(r, b, op, op->op2,
                (op->op1.num & IS_CONSTANT_UNQUALIFIED_IN_NAMESPACE) ? 3 : 2);
            cache(r, b, op, ext, 1);
            break;
        case ZEND_BIND_GLOBAL:
            name_literals(r, b, op, op->op2, 1);
            cache(r, b, op, ext, 1);
            break;
        case ZEND_DEFINED:
        case ZEND_JMP_FRAMELESS:
            name_literals(r, b, op, op->op1, 1);
            cache(r, b, op, ext, 1);
            break;
        case ZEND_DECLARE_CLASS_DELAYED:
        case ZEND_DECLARE_ANON_CLASS:
        case ZEND_DECLARE_CLASS:
            name_literals(r, b, op, op->op1, op->opcode == ZEND_DECLARE_ANON_CLASS ? 1 : 2);
            if (two) name_literals(r, b, op, op->op2, 1);
            if (op->opcode != ZEND_DECLARE_CLASS) cache(r, b, op, ext, 1);
            break;
        case ZEND_DECLARE_FUNCTION:
        case ZEND_DECLARE_CONST:
        case ZEND_DECLARE_ATTRIBUTED_CONST:
        case ZEND_INIT_USER_CALL:
        case ZEND_INIT_PARENT_PROPERTY_HOOK_CALL:
            name_literals(r, b, op, op->op1, 1);
            break;
        case ZEND_DECLARE_LAMBDA_FUNCTION:
        case ZEND_CALLABLE_CONVERT:
            if (ext != UINT32_MAX) {
                cache(r, b, op, ext, op->opcode == ZEND_CALLABLE_CONVERT ? 1 : 2);
            }
            break;
        case ZEND_CALLABLE_CONVERT_PARTIAL:
            name_literals(r, b, op, op->op1, 1);
            cache(r, b, op, ext & ~ZEND_PARTIAL_USES_VARIADIC_PLACEHOLDER, 2);
            if (two && Z_TYPE_P(RT_CONSTANT(op, op->op2)) != IS_ARRAY) {
                invalid(r, b, op, "expected an array of named placeholder positions");
            }
            break;
        case ZEND_ASSIGN_OBJ_OP:
        case ZEND_ASSIGN_OBJ:
        case ZEND_ASSIGN_OBJ_REF:
        case ZEND_FETCH_OBJ_R:
        case ZEND_FETCH_OBJ_W:
        case ZEND_FETCH_OBJ_RW:
        case ZEND_FETCH_OBJ_IS:
        case ZEND_FETCH_OBJ_UNSET:
        case ZEND_FETCH_OBJ_FUNC_ARG:
        case ZEND_UNSET_OBJ:
        case ZEND_PRE_INC_OBJ:
        case ZEND_PRE_DEC_OBJ:
        case ZEND_POST_INC_OBJ:
        case ZEND_POST_DEC_OBJ:
        case ZEND_ISSET_ISEMPTY_PROP_OBJ:
            if (two) {
                name_literals(r, b, op, op->op2, 1);
                uint32_t offset = op->opcode == ZEND_ASSIGN_OBJ_OP ? op[1].extended_value : ext;
                cache(r, b, op, offset & ~ZEND_FETCH_OBJ_FLAGS, 3);
            }
            break;
        case ZEND_ASSIGN_STATIC_PROP_OP:
        case ZEND_ASSIGN_STATIC_PROP:
        case ZEND_ASSIGN_STATIC_PROP_REF:
        case ZEND_FETCH_STATIC_PROP_R:
        case ZEND_FETCH_STATIC_PROP_W:
        case ZEND_FETCH_STATIC_PROP_RW:
        case ZEND_FETCH_STATIC_PROP_IS:
        case ZEND_FETCH_STATIC_PROP_UNSET:
        case ZEND_FETCH_STATIC_PROP_FUNC_ARG:
        case ZEND_UNSET_STATIC_PROP:
        case ZEND_ISSET_ISEMPTY_STATIC_PROP:
        case ZEND_PRE_INC_STATIC_PROP:
        case ZEND_PRE_DEC_STATIC_PROP:
        case ZEND_POST_INC_STATIC_PROP:
        case ZEND_POST_DEC_STATIC_PROP:
            if (one) name_literals(r, b, op, op->op1, 1);
            if (two) name_literals(r, b, op, op->op2, 2);
            if (one || two) {
                uint32_t offset = op->opcode == ZEND_ASSIGN_STATIC_PROP_OP ? op[1].extended_value : ext;
                cache(r, b, op, offset & ~ZEND_FETCH_OBJ_FLAGS, one ? 3 : 1);
            }
            break;
        case ZEND_SEND_VAL:
        case ZEND_SEND_VAL_EX:
        case ZEND_SEND_VAR:
        case ZEND_SEND_VAR_EX:
        case ZEND_SEND_REF:
        case ZEND_SEND_VAR_NO_REF:
        case ZEND_SEND_VAR_NO_REF_EX:
        case ZEND_CHECK_FUNC_ARG:
        case ZEND_SEND_FUNC_ARG:
        case ZEND_SEND_PLACEHOLDER:
            if (two) {
                name_literals(r, b, op, op->op2, 1);
                cache(r, b, op, op->result.num, 2);
            }
            break;
    }
}

static void check_static(zo_reader *r, const zend_op_array *body, const zend_op *op,
        const zend_op_array *owner, uint32_t offset, zend_string *name)
{
    HashTable *vars = owner->static_variables;
    uint32_t index = offset / sizeof(Bucket);
    if (!vars || HT_IS_PACKED(vars) || offset % sizeof(Bucket) || index >= vars->nNumUsed
            || !vars->arData[index].key || !zend_string_equals(vars->arData[index].key, name)) {
        invalid(r, body, op, "static variable offset does not identify the named binding");
    }
}

static void check_body_instruction(zo_reader *r, const zend_op_array *b, const zend_op *op)
{
    uint32_t index = (uint32_t)(op - b->opcodes);
    uint32_t ext = op->extended_value;
    switch (op->opcode) {
        case ZEND_JMP_NULL:
            if ((ext & ZEND_SHORT_CIRCUITING_CHAIN_MASK) > ZEND_SHORT_CIRCUITING_CHAIN_EMPTY) {
                invalid(r, b, op, "invalid nullsafe chain mode");
            }
            break;
        case ZEND_RECV:
        case ZEND_RECV_INIT:
        case ZEND_RECV_VARIADIC:
            if (op->op1.num != index + 1 || EX_VAR_TO_NUM(op->result.var) != index
                    || (op->opcode == ZEND_RECV_VARIADIC
                        ? (!(b->fn_flags & ZEND_ACC_VARIADIC) || index != b->num_args)
                        : index >= b->num_args)) {
                invalid(r, b, op, "parameter position disagrees with the function declaration");
            }
            break;
        case ZEND_VERIFY_RETURN_TYPE:
            if (!(b->fn_flags & ZEND_ACC_HAS_RETURN_TYPE) || !b->arg_info
                    || (op->op1_type == IS_CONST && op->result_type == IS_UNUSED)) {
                invalid(r, b, op, "return type check needs a declaration and a result slot for a literal");
            }
            break;
        case ZEND_GENERATOR_CREATE:
        case ZEND_GENERATOR_RETURN:
        case ZEND_YIELD:
        case ZEND_YIELD_FROM:
            if (!(b->fn_flags & ZEND_ACC_GENERATOR)) invalid(r, b, op, "requires a generator body");
            break;
        case ZEND_FAST_RET:
            if (op->op2.num != UINT32_MAX && op->op2.num >= b->last_try_catch) {
                invalid(r, b, op, "exception region index is out of range");
            }
            break;
        case ZEND_DECLARE_FUNCTION:
        case ZEND_DECLARE_LAMBDA_FUNCTION: {
            if (op->op2.num >= b->num_dynamic_func_defs) {
                invalid(r, b, op, "nested function index is out of range");
            }
            const zend_op_array *child = b->dynamic_func_defs[op->op2.num];
            bool closure = op->opcode == ZEND_DECLARE_LAMBDA_FUNCTION;
            if (!child || !child->function_name || !!(child->fn_flags & ZEND_ACC_CLOSURE) != closure
                    || (!closure && !zend_string_equals_ci(child->function_name,
                        Z_STR_P(RT_CONSTANT(op, op->op1))))) {
                invalid(r, b, op, "nested function declaration does not match its body");
            }
            break;
        }
        case ZEND_BIND_STATIC:
        case ZEND_BIND_INIT_STATIC_OR_JMP: {
            uint32_t offset = ext;
            if (op->opcode == ZEND_BIND_STATIC) {
                offset &= ~(ZEND_BIND_REF | ZEND_BIND_EXPLICIT | ZEND_BIND_IMPLICIT);
            }
            check_static(r, b, op, b, offset, b->vars[EX_VAR_TO_NUM(op->op1.var)]);
            break;
        }
        case ZEND_BIND_LEXICAL: {
            const zend_op *init = op;
            while (init > b->opcodes && init[-1].opcode == ZEND_BIND_LEXICAL) {
                --init;
            }
            if (init == b->opcodes || init[-1].opcode != ZEND_DECLARE_LAMBDA_FUNCTION
                    || init[-1].result.var != op->op1.var) {
                invalid(r, b, op, "lexical binding does not follow its closure declaration");
            }
            check_static(r, b, op, b->dynamic_func_defs[init[-1].op2.num],
                ext & ~(ZEND_BIND_REF | ZEND_BIND_IMPLICIT), b->vars[EX_VAR_TO_NUM(op->op2.var)]);
            break;
        }
        case ZEND_ASSIGN_OP:
        case ZEND_ASSIGN_DIM_OP:
        case ZEND_ASSIGN_OBJ_OP:
        case ZEND_ASSIGN_STATIC_PROP_OP:
            if (ext < ZEND_ADD || ext > ZEND_POW) {
                invalid(r, b, op, "unknown compound assignment operator");
            }
            break;
        case ZEND_CAST:
            if (ext != IS_LONG && ext != IS_DOUBLE && ext != IS_STRING && ext != IS_ARRAY && ext != IS_OBJECT) {
                invalid(r, b, op, "invalid cast type");
            }
            break;
        case ZEND_CONCAT:
        case ZEND_FAST_CONCAT:
            if (op->opcode == ZEND_CONCAT && op->op1_type == IS_CONST && op->op2_type == IS_CONST) {
                invalid(r, b, op, "CONCAT has no two-literal handler");
            }
            if ((op->op1_type == IS_CONST && Z_TYPE_P(RT_CONSTANT(op, op->op1)) != IS_STRING)
                    || (op->op2_type == IS_CONST && Z_TYPE_P(RT_CONSTANT(op, op->op2)) != IS_STRING)) {
                invalid(r, b, op, "expected a string concatenation literal");
            }
            break;
        case ZEND_INCLUDE_OR_EVAL:
            if (ext != ZEND_EVAL && ext != ZEND_INCLUDE && ext != ZEND_INCLUDE_ONCE
                    && ext != ZEND_REQUIRE && ext != ZEND_REQUIRE_ONCE) {
                invalid(r, b, op, "invalid inclusion mode");
            }
            break;
        case ZEND_ROPE_INIT:
        case ZEND_ROPE_ADD:
        case ZEND_ROPE_END: {
            uint32_t pos = op->opcode == ZEND_ROPE_INIT ? op->result.var : op->op1.var;
            uint64_t available = (uint64_t)(b->last_var + b->T - EX_VAR_TO_NUM(pos)) * sizeof(zval);
            uint64_t count = op->opcode == ZEND_ROPE_INIT ? ext : (uint64_t)ext + 1;
            if (!count || count * sizeof(zend_string *) > available
                    || (op->opcode == ZEND_ROPE_ADD && op->result.var != pos)
                    || (op->opcode == ZEND_ROPE_END && op->result.var >= pos
                        && op->result.var - pos < count * sizeof(zend_string *))) {
                invalid(r, b, op, "invalid rope storage");
            }
            if (op->op2_type == IS_CONST) name_literals(r, b, op, op->op2, 1);
            break;
        }
        case ZEND_INIT_ARRAY:
        case ZEND_ADD_ARRAY_ELEMENT:
            if (op->op1_type == IS_CONST && Z_ISUNDEF_P(RT_CONSTANT(op, op->op1))) {
                invalid(r, b, op, "undef cannot be an array element");
            }
            break;
        case ZEND_IN_ARRAY: {
            zval *lookup = RT_CONSTANT(op, op->op2);
            if (Z_TYPE_P(lookup) != IS_ARRAY) invalid(r, b, op, "expected a lookup array");
            if (!ext) {
                zend_string *key;
                ZEND_HASH_FOREACH_STR_KEY(Z_ARR_P(lookup), key) {
                    if (!key) invalid(r, b, op, "non-strict IN_ARRAY requires string keys");
                } ZEND_HASH_FOREACH_END();
            }
            break;
        }
        case ZEND_DECLARE_ATTRIBUTED_CONST:
            if (index + 1 >= b->last || op[1].opcode != ZEND_OP_DATA || op[1].op1_type != IS_CONST
                    || Z_TYPE_P(RT_CONSTANT(&op[1], op[1].op1)) != IS_PTR) {
                invalid(r, b, op, "expected a following attribute-list literal");
            }
            break;
        case ZEND_INIT_PARENT_PROPERTY_HOOK_CALL:
            if (!b->scope || (b->fn_flags & ZEND_ACC_STATIC) || op->op2.num >= ZEND_PROPERTY_HOOK_COUNT) {
                invalid(r, b, op, "invalid property hook in this scope");
            }
            break;
        case ZEND_FUNC_GET_ARGS:
            if (op->op1_type == IS_CONST && (Z_TYPE_P(RT_CONSTANT(op, op->op1)) != IS_LONG
                    || (zend_ulong)Z_LVAL_P(RT_CONSTANT(op, op->op1)) > UINT32_MAX)) {
                invalid(r, b, op, "argument offset must be a nonnegative 32-bit integer");
            }
            break;
        case ZEND_FETCH_R:
        case ZEND_FETCH_W:
        case ZEND_FETCH_RW:
        case ZEND_FETCH_IS:
        case ZEND_FETCH_FUNC_ARG:
        case ZEND_FETCH_UNSET:
        case ZEND_UNSET_VAR:
        case ZEND_ISSET_ISEMPTY_VAR:
            if (op->op1_type == IS_CONST) name_literals(r, b, op, op->op1, 1);
            break;
    }
}

void zo_validate_code(zo_reader *r, zend_op_array *body, uint32_t executable)
{
    uint32_t params = body->num_args + !!(body->fn_flags & ZEND_ACC_VARIADIC);
    if (executable <= params) zo_fail(r, "missing code after parameters");
    for (uint32_t i = 0; i < body->last; ++i) {
        const zend_op *op = &body->opcodes[i];
        const operand_shape *shape = &shapes[op->opcode];
        if (!shape->op1) invalid(r, body, op, "unsupported instruction");
        uint8_t result = op->result_type & ~(IS_SMART_BRANCH_JMPZ | IS_SMART_BRANCH_JMPNZ);
        if (!(shape->op1 & operand_bit(op->op1_type)) || !(shape->op2 & operand_bit(op->op2_type))
                || !(shape->result & operand_bit(result))) {
            invalid(r, body, op, "operand or result type does not match instruction");
        }
        if (i < params && op->opcode != ZEND_RECV && op->opcode != ZEND_RECV_INIT
                && op->opcode != ZEND_RECV_VARIADIC) {
            invalid(r, body, op, "expected parameter receive instruction");
        }
        check_names_and_cache(r, body, op);
        check_body_instruction(r, body, op);
    }
    const zend_op *last = &body->opcodes[executable - 1];
    if (executable < body->last) {
        for (uint32_t i = 0; i < body->last_try_catch; ++i) {
            const zend_try_catch_element *region = &body->try_catch_array[i];
            if (region->catch_op >= executable || region->finally_op >= executable) {
                invalid(r, body, last, "exception handler enters trailing cleanup");
            }
        }
    }
    switch (last->opcode) {
        case ZEND_RETURN:
        case ZEND_RETURN_BY_REF:
        case ZEND_GENERATOR_RETURN:
        case ZEND_THROW:
        case ZEND_FAST_RET:
        case ZEND_JMP:
        case ZEND_MATCH:
        case ZEND_MATCH_ERROR:
        case ZEND_VERIFY_NEVER_TYPE:
            break;
        case ZEND_DO_ICALL:
        case ZEND_DO_UCALL:
            if (executable == body->last) break;
            ZEND_FALLTHROUGH;
        default:
            invalid(r, body, last, "body can fall past its final instruction");
    }
    struct zo_pending_body *entry = zend_arena_alloc(&CG(arena), sizeof(*entry));
    entry->body = body;
    entry->next = r->pending;
    r->pending = entry;
}

typedef struct {
    uint32_t parent;
    uint32_t argc;
    uint32_t placeholders;
    const zend_function *target;
    enum { NO_CALL_CHANGE, OPEN_CALL, CLOSE_CALL } change;
} call_site;

#define NO_CALL UINT32_MAX
#define UNREACHED (UINT32_MAX - 1)

static const zend_function *direct_method(const zend_op_array *b, const zend_op *op)
{
    if (op->opcode == ZEND_INIT_PARENT_PROPERTY_HOOK_CALL) {
        const zend_class_entry *scope = b->scope;
        if (!scope || !(scope->ce_flags & ZEND_ACC_LINKED) || !scope->parent) return NULL;
        zend_property_info *prop = zend_hash_find_ptr(&scope->parent->properties_info,
            Z_STR_P(RT_CONSTANT(op, op->op1)));
        return prop && prop->hooks ? prop->hooks[op->op2.num] : NULL;
    }
    if (op->op2_type != IS_CONST) return NULL;
    const zend_class_entry *ce = NULL;
    if (op->opcode == ZEND_INIT_METHOD_CALL) {
        if (op->op1_type == IS_UNUSED) ce = b->scope;
    } else if (op->op1_type == IS_CONST) {
        zend_string *key = Z_STR_P(RT_CONSTANT(op, op->op1) + 1);
        ce = zend_hash_find_ptr(EG(class_table), key);
    } else if (op->op1_type == IS_UNUSED) {
        if ((op->op1.num & ZEND_FETCH_CLASS_MASK) == ZEND_FETCH_CLASS_SELF) ce = b->scope;
        if ((op->op1.num & ZEND_FETCH_CLASS_MASK) == ZEND_FETCH_CLASS_PARENT
                && b->scope && (b->scope->ce_flags & ZEND_ACC_LINKED)) ce = b->scope->parent;
    }
    if (!ce) return NULL;
    zend_string *name = Z_STR_P(RT_CONSTANT(op, op->op2) + 1);
    return zend_hash_find_ptr(&ce->function_table, name);
}

static void check_declared_class(zo_reader *r, const zend_op_array *b, const zend_op *op)
{
    zval *names = RT_CONSTANT(op, op->op1);
    bool anonymous = op->opcode == ZEND_DECLARE_ANON_CLASS;
    zend_class_entry *ce = zend_hash_find_ptr(EG(class_table), Z_STR(names[anonymous ? 0 : 1]));
    if (!ce && op->opcode == ZEND_DECLARE_CLASS_DELAYED) {
        ce = zend_hash_find_ptr(EG(class_table), Z_STR(names[0]));
    }
    if (!ce || ce->type != ZEND_USER_CLASS || !zend_string_equals(ce->info.user.filename, b->filename)
            || (!!(ce->ce_flags & ZEND_ACC_ANON_CLASS) != anonymous)
            || (!anonymous && !zend_string_equals_ci(ce->name, Z_STR(names[0])))) {
        invalid(r, b, op, "class declaration has no matching class in this unit");
    }
}

static uint32_t freeze_bound_arg(zo_reader *r, const zend_op *send, uint32_t mask)
{
    if (!mask || send->op1_type != IS_CONST || send->op2_type != IS_UNUSED
            || (send->opcode != ZEND_SEND_VAL && send->opcode != ZEND_SEND_VAL_EX
                && send->opcode != ZEND_SEND_USER)) return 0;
    zval *value = RT_CONSTANT(send, send->op1);
    uint32_t position = send->op2.num;
    if (Z_TYPE_P(value) >= IS_OBJECT || !position || position > 32) return 0;
    uint32_t bit = UINT32_C(1) << (position - 1);
    if (!(mask & bit)) return 0;
    zo_freeze_bound_literal(r, value);
    return bit;
}

static void check_partial(zo_reader *r, const zend_op_array *b, uint32_t index,
        const call_site *sites, uint32_t init)
{
    const zend_op *op = &b->opcodes[index];
    HashTable *names = NULL;
    uint32_t mask = op->op2.num;
    if (op->op2_type == IS_CONST) {
        zval *positions = RT_CONSTANT(op, op->op2);
        names = Z_ARRVAL_P(positions);
        mask = Z_EXTRA_P(positions);
    }
    if (!sites[init].placeholders && !(op->extended_value & ZEND_PARTIAL_USES_VARIADIC_PLACEHOLDER)) {
        invalid(r, b, op, "partial application needs a placeholder");
    }
    uint32_t named = 0;
    uint32_t frozen = 0;
    for (uint32_t i = init + 1; i < index; ++i) {
        const zend_op *send = &b->opcodes[i];
        if (sites[i].parent != init) continue;
        frozen |= freeze_bound_arg(r, send, mask);
        if (send->opcode != ZEND_SEND_PLACEHOLDER || send->op2_type != IS_CONST) continue;
        zend_string *name = Z_STR_P(RT_CONSTANT(send, send->op2));
        zval *position = names ? zend_hash_find(names, name) : NULL;
        if (!position || Z_TYPE_P(position) != IS_LONG || Z_LVAL_P(position) != named) {
            invalid(r, b, op, "named placeholder order disagrees with the argument list");
        }
        ++named;
    }
    if (mask & ~frozen) invalid(r, b, op, "bound argument mask refers to a nonconstant argument");
    if (names && named != zend_hash_num_elements(names)) invalid(r, b, op, "surplus named placeholder position");
}

static void init_call_sites(zo_reader *r, const zend_op_array *b, call_site *sites)
{
    for (uint32_t i = 0; i < b->last; ++i) {
        const zend_op *op = &b->opcodes[i];
        call_site *site = &sites[i];
        site->parent = UNREACHED;
        switch (op->opcode) {
            case ZEND_INIT_FCALL:
            case ZEND_INIT_FCALL_BY_NAME:
            case ZEND_INIT_NS_FCALL_BY_NAME:
            case ZEND_INIT_DYNAMIC_CALL:
            case ZEND_INIT_METHOD_CALL:
            case ZEND_INIT_STATIC_METHOD_CALL:
            case ZEND_INIT_PARENT_PROPERTY_HOOK_CALL:
            case ZEND_INIT_USER_CALL:
            case ZEND_NEW:
                site->change = OPEN_CALL;
                site->argc = op->extended_value;
                site->placeholders = 0;
                site->target = NULL;
                if (site->argc > INT_MAX / sizeof(zval) - ZEND_CALL_FRAME_SLOT) {
                    invalid(r, b, op, "argument count exceeds the Zend call frame limit");
                }
                if (op->opcode == ZEND_INIT_FCALL) {
                    zend_string *key = Z_STR_P(RT_CONSTANT(op, op->op2));
                    site->target = zend_hash_find_ptr(EG(function_table), key);
                    if (!site->target) invalid(r, b, op, "direct call target is not declared");
                    uint64_t slots = (uint64_t)ZEND_CALL_FRAME_SLOT + site->argc + site->target->common.T;
                    if (site->target->type == ZEND_USER_FUNCTION) {
                        slots += site->target->op_array.last_var - MIN(site->argc, site->target->common.num_args);
                    }
                    if (slots * sizeof(zval) > op->op1.num || op->op1.num % sizeof(zval)) {
                        invalid(r, b, op, "direct call frame is too small or misaligned");
                    }
                } else if (op->opcode == ZEND_INIT_METHOD_CALL || op->opcode == ZEND_INIT_STATIC_METHOD_CALL
                        || op->opcode == ZEND_INIT_PARENT_PROPERTY_HOOK_CALL) {
                    site->target = direct_method(b, op);
                }
                break;
            case ZEND_DO_ICALL:
            case ZEND_DO_UCALL:
            case ZEND_DO_FCALL:
            case ZEND_DO_FCALL_BY_NAME:
            case ZEND_CALLABLE_CONVERT:
            case ZEND_CALLABLE_CONVERT_PARTIAL:
                site->change = CLOSE_CALL;
                break;
            default:
                site->change = NO_CALL_CHANGE;
                break;
        }
    }
}

static bool call_never_returns(const call_site *site, const zend_op *op)
{
    const zend_function *target = site->target;
    return (op->opcode == ZEND_DO_ICALL || op->opcode == ZEND_DO_UCALL)
        && target && (target->common.fn_flags & ZEND_ACC_HAS_RETURN_TYPE)
        && (ZEND_TYPE_PURE_MASK(target->common.arg_info[-1].type) & MAY_BE_NEVER);
}

static bool raises_exception(uint8_t opcode)
{
    return opcode == ZEND_THROW || opcode == ZEND_MATCH_ERROR || opcode == ZEND_VERIFY_NEVER_TYPE;
}

static const zend_op *jump_target(const zend_op *op)
{
    uint32_t flags = zend_get_opcode_flags(op->opcode);
    if ((ZEND_VM_OP1_FLAGS(flags) & ZEND_VM_OP_MASK) == ZEND_VM_OP_JMP_ADDR) {
        return OP_JMP_ADDR(op, op->op1);
    }
    if ((ZEND_VM_OP2_FLAGS(flags) & ZEND_VM_OP_MASK) == ZEND_VM_OP_JMP_ADDR
            && !(op->opcode == ZEND_CATCH && (op->extended_value & ZEND_LAST_CATCH))) {
        return OP_JMP_ADDR(op, op->op2);
    }
    if ((flags & ZEND_VM_EXT_MASK) == ZEND_VM_EXT_JMP_ADDR) {
        return ZEND_OFFSET_TO_OPLINE(op, op->extended_value);
    }
    return NULL;
}

static void enter_call_path(zo_reader *r, const zend_op_array *b, const zend_op *from,
        call_site *sites, uint32_t *work, uint32_t *count, uint32_t to, uint32_t top, const char *reason)
{
    if (sites[to].parent == UNREACHED) {
        sites[to].parent = top;
        work[(*count)++] = to;
    } else if (sites[to].parent != top) {
        invalid(r, b, from, reason);
    }
}

static void find_call_paths(zo_reader *r, const zend_op_array *b, call_site *sites)
{
    uint32_t *work = safe_emalloc(b->last, sizeof(*work), 0);
    uint32_t count = 0;
    const char *crossed = "jump crosses an unfinished call";
    enter_call_path(r, b, b->opcodes, sites, work, &count, 0, NO_CALL, crossed);
    for (uint32_t i = 0; i < b->last_try_catch; ++i) {
        const zend_try_catch_element *region = &b->try_catch_array[i];
        if (region->catch_op) enter_call_path(r, b, b->opcodes, sites, work, &count, region->catch_op, NO_CALL, crossed);
        if (region->finally_op) enter_call_path(r, b, b->opcodes, sites, work, &count, region->finally_op, NO_CALL, crossed);
    }
    while (count) {
        uint32_t i = work[--count];
        const zend_op *op = &b->opcodes[i];
        uint32_t top = sites[i].parent;
        if (sites[i].change == OPEN_CALL) {
            top = i;
        } else if (sites[i].change == CLOSE_CALL && top != NO_CALL) {
            if (call_never_returns(&sites[top], op)) continue;
            top = sites[top].parent;
        }
        if (raises_exception(op->opcode)) continue;
        if (op->opcode == ZEND_RETURN || op->opcode == ZEND_RETURN_BY_REF
                || op->opcode == ZEND_GENERATOR_RETURN || op->opcode == ZEND_FAST_RET) {
            if (top != NO_CALL) invalid(r, b, &b->opcodes[top], "unfinished call");
            continue;
        }
        const zend_op *target = jump_target(op);
        if (target) enter_call_path(r, b, op, sites, work, &count, target - b->opcodes, top, crossed);
        if (op->opcode == ZEND_MATCH || op->opcode == ZEND_SWITCH_LONG || op->opcode == ZEND_SWITCH_STRING) {
            zval *offset;
            ZEND_HASH_FOREACH_VAL(Z_ARRVAL_P(RT_CONSTANT(op, op->op2)), offset) {
                target = ZEND_OFFSET_TO_OPLINE(op, Z_LVAL_P(offset));
                enter_call_path(r, b, op, sites, work, &count, target - b->opcodes, top,
                    "table target crosses an unfinished call");
            } ZEND_HASH_FOREACH_END();
        }
        if (op->opcode != ZEND_JMP && op->opcode != ZEND_MATCH && i + 1 < b->last) {
            enter_call_path(r, b, op, sites, work, &count, i + 1, top, crossed);
        }
    }
    efree(work);
}

static void validate_calls_in_body(zo_reader *r, const zend_op_array *b)
{
    const zend_string *parent_body = r->body;
    r->body = b->function_name;
    call_site *sites = safe_emalloc(b->last, sizeof(*sites), 0);
    init_call_sites(r, b, sites);
    find_call_paths(r, b, sites);
    uint32_t top = NO_CALL;
    for (uint32_t i = 0; i < b->last; ++i) {
        const zend_op *op = &b->opcodes[i];
        call_site *site = &sites[i];
        /* A source-compiled throw expression may retain unreachable sends and a call completion. */
        if (site->parent == UNREACHED) site->parent = top;
        else top = site->parent;
        if (site->change == OPEN_CALL) {
            top = i;
            continue;
        }
        switch (op->opcode) {
            case ZEND_SEND_VAL:
            case ZEND_SEND_VAL_EX:
            case ZEND_SEND_VAR:
            case ZEND_SEND_VAR_EX:
            case ZEND_SEND_REF:
            case ZEND_SEND_VAR_NO_REF:
            case ZEND_SEND_VAR_NO_REF_EX:
            case ZEND_SEND_FUNC_ARG:
            case ZEND_SEND_USER:
            case ZEND_CHECK_FUNC_ARG:
            case ZEND_SEND_PLACEHOLDER:
                if (top == UINT32_MAX) invalid(r, b, op, "argument without an open call");
                if (op->op2_type == IS_UNUSED && (op->op2.num == 0 || op->op2.num > sites[top].argc)) {
                    invalid(r, b, op, "argument position exceeds the open call frame");
                }
                if (op->opcode == ZEND_SEND_PLACEHOLDER) ++sites[top].placeholders;
                break;
            case ZEND_SEND_UNPACK:
            case ZEND_SEND_ARRAY:
            case ZEND_CHECK_UNDEF_ARGS:
            case ZEND_FETCH_FUNC_ARG:
            case ZEND_FETCH_DIM_FUNC_ARG:
            case ZEND_FETCH_OBJ_FUNC_ARG:
            case ZEND_FETCH_STATIC_PROP_FUNC_ARG:
                if (top == UINT32_MAX) invalid(r, b, op, "instruction needs an open call");
                break;
            case ZEND_DO_ICALL:
            case ZEND_DO_UCALL:
            case ZEND_DO_FCALL:
            case ZEND_DO_FCALL_BY_NAME:
            case ZEND_CALLABLE_CONVERT:
            case ZEND_CALLABLE_CONVERT_PARTIAL: {
                if (top == UINT32_MAX) invalid(r, b, op, "call completion without an open call");
                const zend_function *target = sites[top].target;
                if (op->opcode == ZEND_CALLABLE_CONVERT_PARTIAL) {
                    check_partial(r, b, i, sites, top);
                } else if (sites[top].placeholders) {
                    invalid(r, b, op, "placeholder sent to an ordinary call");
                }
                if (op->opcode == ZEND_DO_ICALL && (!target || target->type != ZEND_INTERNAL_FUNCTION)) {
                    invalid(r, b, op, "internal call needs a known internal function");
                }
                if (op->opcode == ZEND_DO_UCALL && (!target || target->type != ZEND_USER_FUNCTION)) {
                    invalid(r, b, op, "user call needs a known user function");
                }
                if (i == b->last - 1 && !call_never_returns(&sites[top], op)) {
                    invalid(r, b, op, "body can fall past its final call");
                }
                top = i == b->last - 1 ? NO_CALL : sites[top].parent;
                break;
            }
            case ZEND_DECLARE_CLASS:
            case ZEND_DECLARE_CLASS_DELAYED:
            case ZEND_DECLARE_ANON_CLASS:
                check_declared_class(r, b, op);
                break;
        }
    }
    if (top != NO_CALL && !raises_exception(b->opcodes[b->last - 1].opcode)) {
        invalid(r, b, &b->opcodes[top], "unfinished call");
    }

    /* Branches may skip whole calls or branch inside an argument expression. */
    for (uint32_t i = 0; i < b->last; ++i) {
        const zend_op *op = &b->opcodes[i];
        const zend_op *target = jump_target(op);
        if (target && sites[i].parent != sites[target - b->opcodes].parent) {
            invalid(r, b, op, "jump crosses an unfinished call");
        }
        if (op->opcode == ZEND_MATCH || op->opcode == ZEND_SWITCH_LONG || op->opcode == ZEND_SWITCH_STRING) {
            zval *offset;
            ZEND_HASH_FOREACH_VAL(Z_ARRVAL_P(RT_CONSTANT(op, op->op2)), offset) {
                target = ZEND_OFFSET_TO_OPLINE(op, Z_LVAL_P(offset));
                if (sites[i].parent != sites[target - b->opcodes].parent) {
                    invalid(r, b, op, "table target crosses an unfinished call");
                }
            } ZEND_HASH_FOREACH_END();
        }
    }
    efree(sites);
    r->body = parent_body;
}

void zo_validate_unit_calls(zo_reader *r)
{
    for (struct zo_pending_body *entry = r->pending; entry; entry = entry->next) {
        validate_calls_in_body(r, entry->body);
    }
}
