--TEST--
The runtime halt offset agrees with the compiled offset
--FILE--
<?php
var_dump(constant("__COMPILER_HALT_OFFSET__") === __COMPILER_HALT_OFFSET__);
__halt_compiler();payload
--EXPECT--
bool(true)
