PHP_ARG_ENABLE([zend_opcodes],
  [whether to build the Zend opcode printer and loader],
  [AS_HELP_STRING([--enable-zend-opcodes], [Build the Zend opcode printer and loader])],
  [yes])

if test "$PHP_ZEND_OPCODES" != "no"; then
  PHP_NEW_EXTENSION([zend_opcodes],
    [zend_opcodes.c unit.c body.c instructions.c validate.c text.c attributes.c constant_ast.c enums.c],
    [$ext_shared])
fi
