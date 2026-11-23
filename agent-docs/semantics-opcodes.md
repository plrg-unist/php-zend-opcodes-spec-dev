# Opcode assignments for the proposed milestones

Each name in the [pinned opcode syntax](../spec/syntax.watsup) has one initial assignment in the [roadmap](semantics-roadmap.md). Introduce the supported operand cases there and extend earlier instructions as later runtime types and behaviors enter. For example, object iteration extends the array foreach rules, and destructors can affect earlier assignment, `FREE` and exception cleanup. The groups cover 212 named opcodes.

## 1. Executable main

```text
NOP ASSIGN PRE_INC IS_SMALLER JMP JMPZ JMPNZ ECHO RETURN
```

## 2. Scalars and control flow

2A covers the scalar cases of these instructions, the string cast, and concatenating compound assignment. It also extends the main-frame instructions from milestone 1.

```text
BOOL BOOL_NOT BOOL_XOR TYPE_CHECK GET_TYPE STRLEN CHECK_VAR QM_ASSIGN COPY_TMP FREE
CAST CONCAT FAST_CONCAT ASSIGN_OP ROPE_INIT ROPE_ADD ROPE_END
```

2B and 2C introduce the following instructions and extend earlier ones to more operands and operators.

```text
ADD SUB MUL DIV MOD SL SR BW_OR BW_AND BW_XOR POW BW_NOT IS_IDENTICAL IS_NOT_IDENTICAL
IS_EQUAL IS_NOT_EQUAL IS_SMALLER_OR_EQUAL PRE_DEC POST_INC POST_DEC JMPZ_EX JMPNZ_EX CASE
JMP_SET COALESCE SPACESHIP SWITCH_LONG SWITCH_STRING MATCH CASE_STRICT JMP_NULL
```

## 3. Arrays, variables and references

```text
ASSIGN_DIM ASSIGN_DIM_OP ASSIGN_REF INIT_ARRAY ADD_ARRAY_ELEMENT UNSET_VAR UNSET_DIM
FE_RESET_R FE_FETCH_R FETCH_R FETCH_DIM_R FETCH_W FETCH_DIM_W FETCH_RW FETCH_DIM_RW FETCH_IS
FETCH_DIM_IS FETCH_UNSET FETCH_DIM_UNSET FETCH_LIST_R ISSET_ISEMPTY_VAR
ISSET_ISEMPTY_DIM_OBJ FE_RESET_RW FE_FETCH_RW FE_FREE OP_DATA MAKE_REF ADD_ARRAY_UNPACK
UNSET_CV ISSET_ISEMPTY_CV FETCH_LIST_W SEPARATE BIND_GLOBAL FETCH_GLOBALS IN_ARRAY COUNT
ARRAY_KEY_EXISTS DECLARE_CONST FETCH_CONSTANT DEFINED
```

## 4. Calls, closures and standard streams

```text
SEND_VAR_NO_REF_EX INIT_FCALL_BY_NAME DO_FCALL INIT_FCALL RECV RECV_INIT SEND_VAL
SEND_VAR_EX SEND_REF INIT_NS_FCALL_BY_NAME FETCH_FUNC_ARG FETCH_DIM_FUNC_ARG CHECK_FUNC_ARG
SEND_VAR_NO_REF RETURN_BY_REF SEND_VAL_EX SEND_VAR INIT_USER_CALL SEND_ARRAY SEND_USER
VERIFY_RETURN_TYPE INIT_DYNAMIC_CALL DO_ICALL DO_UCALL DO_FCALL_BY_NAME DECLARE_FUNCTION
DECLARE_LAMBDA_FUNCTION RECV_VARIADIC SEND_UNPACK BIND_LEXICAL BIND_STATIC SEND_FUNC_ARG
FUNC_NUM_ARGS FUNC_GET_ARGS CHECK_UNDEF_ARGS VERIFY_NEVER_TYPE CALLABLE_CONVERT
BIND_INIT_STATIC_OR_JMP FRAMELESS_ICALL_0 FRAMELESS_ICALL_1 FRAMELESS_ICALL_2
FRAMELESS_ICALL_3 JMP_FRAMELESS TYPE_ASSERT ASSERT_CHECK
```

## 5. Objects and class binding

```text
ASSIGN_OBJ ASSIGN_STATIC_PROP ASSIGN_OBJ_OP ASSIGN_STATIC_PROP_OP ASSIGN_OBJ_REF
ASSIGN_STATIC_PROP_REF PRE_INC_STATIC_PROP PRE_DEC_STATIC_PROP POST_INC_STATIC_PROP
POST_DEC_STATIC_PROP NEW UNSET_OBJ FETCH_OBJ_R FETCH_OBJ_W FETCH_OBJ_RW FETCH_OBJ_IS
FETCH_OBJ_FUNC_ARG FETCH_OBJ_UNSET FETCH_CLASS CLONE INIT_METHOD_CALL
INIT_STATIC_METHOD_CALL PRE_INC_OBJ PRE_DEC_OBJ POST_INC_OBJ POST_DEC_OBJ INSTANCEOF
DECLARE_CLASS DECLARE_CLASS_DELAYED DECLARE_ANON_CLASS ISSET_ISEMPTY_PROP_OBJ
FETCH_CLASS_NAME FETCH_STATIC_PROP_R FETCH_STATIC_PROP_W FETCH_STATIC_PROP_RW
FETCH_STATIC_PROP_IS FETCH_STATIC_PROP_FUNC_ARG FETCH_STATIC_PROP_UNSET UNSET_STATIC_PROP
ISSET_ISEMPTY_STATIC_PROP FETCH_CLASS_CONSTANT FETCH_THIS ISSET_ISEMPTY_THIS GET_CLASS
GET_CALLED_CLASS
```

## 6. Exceptions and runtime diagnostics

```text
BEGIN_SILENCE END_SILENCE CATCH THROW HANDLE_EXCEPTION DISCARD_EXCEPTION FAST_CALL FAST_RET
MATCH_ERROR
```

## 7. Advanced declarations and implicit calls

```text
INIT_PARENT_PROPERTY_HOOK_CALL DECLARE_ATTRIBUTED_CONST CALL_TRAMPOLINE
CALLABLE_CONVERT_PARTIAL SEND_PLACEHOLDER
```

## 8. Lifetime and request shutdown

```text
TICKS
```

## 9. Generators and fibers

```text
GENERATOR_CREATE YIELD GENERATOR_RETURN YIELD_FROM
```

## Fixed execution boundary

```text
INCLUDE_OR_EVAL USER_OPCODE EXT_STMT EXT_FCALL_BEGIN EXT_FCALL_END EXT_NOP
```

An executed `INCLUDE_OR_EVAL` crosses the source-loading boundary. `USER_OPCODE` requires a native opcode hook outside the pinned runtime. With no instrumentation callbacks, `EXT_STMT`, `EXT_FCALL_BEGIN` and `EXT_FCALL_END` have no program-visible effect. `EXT_NOP` advances to the next instruction. [Zend handlers](../../php-src/Zend/zend_vm_def.h#L7997)

`OP_DATA` supplies operands to an owning instruction, including some frameless calls. `HANDLE_EXCEPTION` and `CALL_TRAMPOLINE` are engine-created dispatch points whose effects belong to unwinding and magic method calls. `TYPE_ASSERT` checks a builtin argument in compiler-lowered code. `ASSERT_CHECK` controls evaluation of an assertion. Local tick callbacks and fibers can execute PHP code inside the agreed scope. [Argument checks](../../php-src/Zend/zend_vm_def.h#L8959) · [Exception dispatch](../../php-src/Zend/zend_vm_def.h#L8253) · [Magic calls](../../php-src/Zend/zend_vm_def.h#L9092)
