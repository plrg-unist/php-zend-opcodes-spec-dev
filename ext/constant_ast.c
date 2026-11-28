#include "opcodes.h"
#include "Zend/zend_ast.h"
#include "Zend/zend_arena.h"
#include "Zend/zend_multiply.h"
#include "Zend/zend_partial.h"
#include "Zend/zend_vm_opcodes.h"

static const struct {
    zend_ast_kind kind;
    const char *opening;
} nodes[] = {
    {ZEND_AST_ZVAL, "value("}, {ZEND_AST_CONSTANT, "constant("},
    {ZEND_AST_CLASS_CONST, "class_constant("}, {ZEND_AST_CLASS_NAME, "class_name("},
    {ZEND_AST_CONSTANT_CLASS, "current_class("}, {ZEND_AST_BINARY_OP, "binary("},
    {ZEND_AST_UNARY_OP, "unary("}, {ZEND_AST_UNARY_PLUS, "plus("},
    {ZEND_AST_UNARY_MINUS, "minus("}, {ZEND_AST_CAST, "cast("},
    {ZEND_AST_GREATER, "greater("}, {ZEND_AST_GREATER_EQUAL, "greater_equal("},
    {ZEND_AST_AND, "and("}, {ZEND_AST_OR, "or("}, {ZEND_AST_COALESCE, "coalesce("},
    {ZEND_AST_CONDITIONAL, "conditional("}, {ZEND_AST_DIM, "dimension("},
    {ZEND_AST_PROP, "property("}, {ZEND_AST_NULLSAFE_PROP, "nullsafe_property("},
    {ZEND_AST_ARRAY, "array("}, {ZEND_AST_ARRAY_ELEM, "element("},
    {ZEND_AST_UNPACK, "unpack("}, {ZEND_AST_NEW, "new("},
    {ZEND_AST_ARG_LIST, "arguments("}, {ZEND_AST_NAMED_ARG, "named("},
    {ZEND_AST_OP_ARRAY, "closure("}, {ZEND_AST_CALL, "call("},
    {ZEND_AST_STATIC_CALL, "static_call("}, {ZEND_AST_CALLABLE_CONVERT, "callable("},
    {ZEND_AST_PLACEHOLDER_ARG, "placeholder("}
};

static bool valid_node_attr(zend_ast_kind kind, uint32_t attr)
{
    switch (kind) {
        case ZEND_AST_CONSTANT:
            return attr == 0 || attr == IS_CONSTANT_UNQUALIFIED_IN_NAMESPACE;
        case ZEND_AST_BINARY_OP:
            switch (attr) {
                case ZEND_ADD: case ZEND_SUB: case ZEND_MUL: case ZEND_POW:
                case ZEND_DIV: case ZEND_MOD: case ZEND_SL: case ZEND_SR:
                case ZEND_CONCAT: case ZEND_IS_IDENTICAL: case ZEND_IS_NOT_IDENTICAL:
                case ZEND_IS_EQUAL: case ZEND_IS_NOT_EQUAL: case ZEND_IS_SMALLER:
                case ZEND_IS_SMALLER_OR_EQUAL: case ZEND_SPACESHIP: case ZEND_BW_OR:
                case ZEND_BW_AND: case ZEND_BW_XOR: case ZEND_BOOL_XOR:
                    return true;
                default: return false;
            }
        case ZEND_AST_UNARY_OP:
            return attr == ZEND_BW_NOT || attr == ZEND_BOOL_NOT;
        case ZEND_AST_CAST:
            return attr == _IS_BOOL || attr == IS_LONG || attr == IS_DOUBLE ||
                attr == IS_STRING || attr == IS_ARRAY || attr == IS_OBJECT;
        case ZEND_AST_CLASS_CONST:
            return attr == ZEND_FETCH_CLASS_EXCEPTION;
        case ZEND_AST_CLASS_NAME:
            return attr == ZEND_FETCH_CLASS_SELF || attr == ZEND_FETCH_CLASS_PARENT;
        case ZEND_AST_ARRAY:
            return attr == 0 || attr == ZEND_ARRAY_SYNTAX_LONG || attr == ZEND_ARRAY_SYNTAX_SHORT;
        case ZEND_AST_CONDITIONAL: return attr <= ZEND_PARENTHESIZED_CONDITIONAL;
        case ZEND_AST_DIM: return attr <= ZEND_DIM_IS;
        case ZEND_AST_ARG_LIST: return attr <= 1;
        case ZEND_AST_PLACEHOLDER_ARG: return attr <= ZEND_PLACEHOLDER_VARIADIC;
        case ZEND_AST_ZVAL: return attr <= UINT16_MAX;
        default: return attr == 0;
    }
}

void zo_put_expression(zo_writer *writer, zend_ast *ast)
{
    smart_str *out = writer->out;
    if (!ast) {
        smart_str_appends(out, "none");
        return;
    }
    size_t index = 0;
    while (index < sizeof(nodes) / sizeof(*nodes) && nodes[index].kind != ast->kind) {
        index++;
    }
    uint32_t attr = ast->attr;
    if (ast->kind == ZEND_AST_CALLABLE_CONVERT) attr &= ~ZEND_PARTIAL_CACHEABLE_IN_SHM;
    if (index == sizeof(nodes) / sizeof(*nodes) || !valid_node_attr(ast->kind, attr)) {
        zo_unsupported("constant expression node %u with flags %u", ast->kind, ast->attr);
    }
    smart_str_appends(out, nodes[index].opening);
    smart_str_append_printf(out, "%u %u", attr, zend_ast_get_lineno(ast));
    if (ast->kind == ZEND_AST_ZVAL || ast->kind == ZEND_AST_CONSTANT) {
        smart_str_appendc(out, ' ');
        zo_put_value(writer, &((zend_ast_zval *)ast)->val);
    } else if (ast->kind == ZEND_AST_OP_ARRAY) {
        smart_str_appendc(out, '\n');
        zo_print_body(writer, zend_ast_get_op_array(ast)->op_array);
    } else if (ast->kind == ZEND_AST_CALLABLE_CONVERT) {
        zend_ast_fcc *fcc = (zend_ast_fcc *)ast;
        smart_str_appendc(out, ' ');
        if (fcc->filename) zo_put_string(out, fcc->filename);
        else smart_str_appends(out, "none");
        smart_str_appendc(out, ' ');
        if (fcc->name) zo_put_string(out, fcc->name);
        else smart_str_appends(out, "none");
        smart_str_appendc(out, ' ');
        zo_put_expression(writer, fcc->args);
    } else {
        uint32_t count;
        zend_ast **children;
        if (zend_ast_is_list(ast)) {
            zend_ast_list *list = zend_ast_get_list(ast);
            count = list->children;
            children = list->child;
            smart_str_append_printf(out, " %u", count);
        } else {
            count = zend_ast_get_num_children(ast);
            children = ast->child;
        }
        for (uint32_t i = 0; i < count; i++) {
            smart_str_appendc(out, ' ');
            zo_put_expression(writer, children[i]);
        }
    }
    smart_str_appendc(out, ')');
}

void zo_put_ast(zo_writer *writer, const zval *value)
{
    smart_str *out = writer->out;
    smart_str_appends(out, "ast(");
    zo_put_expression(writer, Z_ASTVAL_P(value));
    smart_str_appendc(out, ')');
}

static bool string_node(zend_ast *node)
{
    return node && node->kind == ZEND_AST_ZVAL && Z_TYPE_P(zend_ast_get_zval(node)) == IS_STRING;
}

static bool expression_node(zend_ast *node)
{
    return node && node->kind != ZEND_AST_ARRAY_ELEM && node->kind != ZEND_AST_UNPACK &&
        node->kind != ZEND_AST_ARG_LIST && node->kind != ZEND_AST_NAMED_ARG &&
        node->kind != ZEND_AST_CALLABLE_CONVERT && node->kind != ZEND_AST_PLACEHOLDER_ARG;
}

static bool class_reference(zend_ast *node)
{
    return string_node(node) && (node->attr == 0 ||
        node->attr == (ZEND_FETCH_CLASS_SELF << ZEND_CONST_EXPR_NEW_FETCH_TYPE_SHIFT) ||
        node->attr == (ZEND_FETCH_CLASS_PARENT << ZEND_CONST_EXPR_NEW_FETCH_TYPE_SHIFT));
}

static bool arglist_has_placeholders(zend_ast_list *list)
{
    for (uint32_t i = 0; i < list->children; i++) {
        zend_ast *arg = list->child[i];
        if (arg->kind == ZEND_AST_NAMED_ARG) arg = arg->child[1];
        if (arg->kind == ZEND_AST_PLACEHOLDER_ARG) return true;
    }
    return false;
}

static bool valid_child(zend_ast_kind parent, uint32_t index, zend_ast *child)
{
    switch (parent) {
        case ZEND_AST_ARRAY:
            return child && (child->kind == ZEND_AST_ARRAY_ELEM || child->kind == ZEND_AST_UNPACK);
        case ZEND_AST_CLASS_NAME:
            return !child;
        case ZEND_AST_ARRAY_ELEM: case ZEND_AST_CONDITIONAL:
            return (!child && index == 1) || expression_node(child);
        case ZEND_AST_CLASS_CONST:
            return index == 0 ? string_node(child) : expression_node(child);
        case ZEND_AST_NAMED_ARG:
            if (index == 0) return string_node(child);
            return expression_node(child) || (child && child->kind == ZEND_AST_PLACEHOLDER_ARG && !child->attr);
        case ZEND_AST_NEW:
            if (index == 1) return child && child->kind == ZEND_AST_ARG_LIST &&
                !arglist_has_placeholders(zend_ast_get_list(child));
            return class_reference(child);
        case ZEND_AST_CALL:
            if (index == 1) return child && child->kind == ZEND_AST_CALLABLE_CONVERT;
            return string_node(child) && child->attr <= ZEND_NAME_RELATIVE;
        case ZEND_AST_STATIC_CALL:
            if (index == 2) return child && child->kind == ZEND_AST_CALLABLE_CONVERT;
            return index == 0 ? class_reference(child) : string_node(child);
        case ZEND_AST_ARG_LIST:
            return expression_node(child) || (child &&
                (child->kind == ZEND_AST_NAMED_ARG || child->kind == ZEND_AST_PLACEHOLDER_ARG));
        default:
            return expression_node(child);
    }
}

static zend_ast *get_node(zo_reader *r, zend_arena **arena, bool allow_dynamic)
{
    if (zo_take(r, "none")) return NULL;
    size_t index = 0;
    while (index < sizeof(nodes) / sizeof(*nodes) && !zo_take(r, nodes[index].opening)) {
        index++;
    }
    if (index == sizeof(nodes) / sizeof(*nodes)) zo_fail(r, "unknown constant expression node");
    zend_ast_kind kind = nodes[index].kind;
    uint32_t attr = zo_u32(r);
    uint32_t line = zo_u32(r);
    if (!valid_node_attr(kind, attr)) zo_fail(r, "invalid constant expression flags %u", attr);
    if (!allow_dynamic && (kind == ZEND_AST_NEW || (kind == ZEND_AST_CAST && attr == IS_OBJECT))) {
        zo_fail(r, "dynamic object creation in constant expression");
    }
    zend_ast *node;
    if (kind == ZEND_AST_ZVAL || kind == ZEND_AST_CONSTANT) {
        zend_ast_zval *leaf = zend_arena_alloc(arena, sizeof(*leaf));
        node = (zend_ast *)leaf;
        if (kind == ZEND_AST_CONSTANT) {
            zend_string *name = zo_string(r);
            if (!ZSTR_LEN(name) || memchr(ZSTR_VAL(name), 0, ZSTR_LEN(name))) {
                zo_fail(r, "invalid constant name");
            }
            ZVAL_STR(&leaf->val, name);
        } else {
            zo_get_value(r, &leaf->val);
            if (Z_ISUNDEF(leaf->val)) zo_fail(r, "undefined value in a constant expression");
        }
        Z_LINENO(leaf->val) = line;
    } else if (kind == ZEND_AST_OP_ARRAY) {
        if (!r->filename) zo_fail(r, "invalid closure default context");
        zo_eol(r);
        zend_ast_op_array *closure = zend_arena_alloc(arena, sizeof(*closure));
        closure->op_array = zend_arena_alloc(&CG(arena), sizeof(zend_op_array));
        r->in_ast = false;
        zo_load_body(r, closure->op_array, r->filename, r->scope);
        r->in_ast = true;
        if (!closure->op_array->function_name ||
            (closure->op_array->fn_flags & (ZEND_ACC_CLOSURE | ZEND_ACC_STATIC)) !=
                (ZEND_ACC_CLOSURE | ZEND_ACC_STATIC)) {
            zo_fail(r, "constant closure requires a named static closure body");
        }
        closure->lineno = line;
        node = (zend_ast *)closure;
    } else if (kind == ZEND_AST_CALLABLE_CONVERT) {
        zend_ast_fcc *fcc = zend_arena_alloc(arena, sizeof(*fcc));
        fcc->lineno = line;
        fcc->filename = zo_take(r, "none") ? NULL : zo_string(r);
        fcc->name = zo_take(r, "none") ? NULL : zo_string(r);
        fcc->args = get_node(r, arena, allow_dynamic);
        if (!fcc->args || fcc->args->kind != ZEND_AST_ARG_LIST) {
            zo_fail(r, "expected arguments for a callable conversion");
        }
        zend_ast_list *args = zend_ast_get_list(fcc->args);
        if (!arglist_has_placeholders(args)) zo_fail(r, "expected a callable placeholder");
        bool full = args->children == 1 && args->child[0]->kind == ZEND_AST_PLACEHOLDER_ARG &&
            args->child[0]->attr == ZEND_PLACEHOLDER_VARIADIC;
        if (full != (fcc->filename == NULL) || full != (fcc->name == NULL)) {
            zo_fail(r, "inconsistent partial application name or filename");
        }
        ZEND_MAP_PTR_INIT(fcc->fptr, NULL);
        node = (zend_ast *)fcc;
    } else {
        zend_ast **children;
        uint32_t count;
        if (kind == ZEND_AST_ARRAY || kind == ZEND_AST_ARG_LIST) {
            count = zo_u32(r);
            bool overflow;
            size_t size = zend_safe_address(count, sizeof(zend_ast *), offsetof(zend_ast_list, child), &overflow);
            if (overflow || count > (size_t)(r->end - r->p) / sizeof("none")) {
                zo_fail(r, "constant expression list exceeds the input");
            }
            zend_ast_list *list = zend_arena_alloc(arena, size);
            list->children = count;
            node = (zend_ast *)list;
            children = list->child;
        } else {
            count = kind >> ZEND_AST_NUM_CHILDREN_SHIFT;
            node = zend_arena_alloc(arena, zend_ast_size(count));
            children = node->child;
        }
        node->lineno = line;
        bool named = false;
        for (uint32_t i = 0; i < count; i++) {
            children[i] = get_node(r, arena, allow_dynamic);
            if (!valid_child(kind, i, children[i])) zo_fail(r, "invalid constant expression child");
            if (kind == ZEND_AST_ARG_LIST) {
                if (children[i]->kind == ZEND_AST_NAMED_ARG) {
                    named = true;
                } else if (children[i]->kind == ZEND_AST_PLACEHOLDER_ARG && children[i]->attr) {
                    if (i + 1 != count) zo_fail(r, "variadic placeholder must be last");
                } else if (named) {
                    zo_fail(r, "positional constructor argument after named argument");
                }
            }
        }
        if (kind == ZEND_AST_ARG_LIST && named != (bool)attr) {
            zo_fail(r, "inconsistent named constructor argument flag");
        }
    }
    node->kind = kind;
    node->attr = (zend_ast_attr)attr;
    zo_expect(r, ")");
    return node;
}

zend_ast_ref *zo_get_expression(zo_reader *r, bool allow_dynamic)
{
    if (r->in_ast) zo_fail(r, "a constant-expression value cannot contain a deferred zval");
    r->in_ast = true;
    zend_arena *arena = zend_arena_create(1024);
    zend_ast *ast = get_node(r, &arena, allow_dynamic);
    zend_ast_ref *ref = NULL;
    if (ast) {
        if (!expression_node(ast)) zo_fail(r, "expected a constant expression");
        ref = zend_ast_copy(ast);
        zend_ast_destroy(ast);
    }
    zend_arena_destroy(arena);
    r->in_ast = false;
    return ref;
}

void zo_get_ast(zo_reader *r, zval *value)
{
    zo_expect(r, "(");
    zend_ast_ref *ref = zo_get_expression(r, true);
    if (!ref) zo_fail(r, "expected a constant expression");
    ZVAL_AST(value, ref);
    zo_expect(r, ")");
}
