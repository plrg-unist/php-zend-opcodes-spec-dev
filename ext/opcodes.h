#ifndef PHP_SPEC_OPCODES_H
#define PHP_SPEC_OPCODES_H

#include "php.h"
#include "Zend/zend_smart_str.h"

#if PHP_VERSION_ID != 80600
#error "Build zend_opcodes with the pinned PHP 8.6.0 development headers"
#endif

typedef struct {
    smart_str *out;
    HashTable *array_offsets;
    HashTable *array_labels;
} zo_writer;

typedef struct {
    const char *p;
    const char *end;
    zend_string *path;
    uint32_t line;
    const zend_string *body;
    bool in_ast;
    zend_string *filename;
    zend_class_entry *scope;
    struct zo_pending_body *pending;
    HashTable *array_bindings;
    HashTable *deferred_array_bindings;
} zo_reader;

ZEND_NORETURN void zo_fail(zo_reader *r, const char *format, ...) ZEND_ATTRIBUTE_FORMAT(printf, 2, 3);
ZEND_NORETURN void zo_unsupported(const char *format, ...) ZEND_ATTRIBUTE_FORMAT(printf, 1, 2);
bool zo_take(zo_reader *r, const char *token);
void zo_expect(zo_reader *r, const char *token);
void zo_eol(zo_reader *r);
uint32_t zo_u32(zo_reader *r);
zend_long zo_long(zo_reader *r);
zend_string *zo_string(zo_reader *r);
void zo_put_string(smart_str *out, const zend_string *s);
void zo_get_value(zo_reader *r, zval *value);
bool zo_get_pool_value(zo_reader *r, zval *value);
zend_array *zo_get_static_table(zo_reader *r);
void zo_freeze_literal_array(zval *value);
void zo_intern_literal_string(zval *value);
zend_result zo_release_immutable_asts(void);
void zo_put_value(zo_writer *writer, const zval *value);
void zo_put_table(zo_writer *writer, const HashTable *table, const zend_op_array *body, uint32_t owner);
uint32_t zo_absolute_target(const zend_op_array *body, uint32_t index, zend_long offset);
void zo_insert_array_labels(zo_writer *writer);
void zo_put_ast(zo_writer *writer, const zval *value);
void zo_get_ast(zo_reader *r, zval *value);
void zo_put_expression(zo_writer *writer, zend_ast *ast);
zend_ast_ref *zo_get_expression(zo_reader *r, bool allow_dynamic);
void zo_put_attributes(zo_writer *writer, const HashTable *attrs);
HashTable *zo_get_attributes(zo_reader *r, uint32_t max_offset);
void zo_put_enum(smart_str *out, const zend_class_entry *ce);
void zo_get_enum(zo_reader *r, zend_class_entry *ce);
void zo_put_enum_case(zo_writer *writer, zend_string *name, const zend_class_constant *constant);
void zo_get_enum_case(zo_reader *r, zend_class_entry *ce, uint32_t expected_id);
void zo_check_enum(zo_reader *r, const zend_class_entry *ce);

void zo_print_literal(zo_writer *writer, const zend_op_array *body, uint32_t index);
void zo_print_instructions(smart_str *out, const zend_op_array *body);
void zo_read_instructions(zo_reader *r, zend_op_array *body, const bool *immutable_literals);
void zo_validate_code(zo_reader *r, zend_op_array *body, uint32_t executable);
void zo_validate_unit_calls(zo_reader *r);

void zo_print_body(zo_writer *writer, const zend_op_array *body);
void zo_load_body(zo_reader *r, zend_op_array *body, zend_string *filename, zend_class_entry *scope);
void zo_put_type(smart_str *out, zend_type type);
zend_type zo_get_type(zo_reader *r);

void zo_apply_constants(zend_string *halt_key, zend_long halt_offset);
void zo_restore_constants(void);
uint8_t zo_armed_autoglobals(void);
void zo_print_unit(smart_str *out, const zend_op_array *main, uint32_t functions_start,
    uint32_t classes_start, uint8_t autoglobals_before);
zend_op_array *zo_load_unit(zo_reader *r);

#endif
