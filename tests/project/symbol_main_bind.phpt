--TEST--
Consecutive global bindings preserve aliases across unset and rebind in main
--FILE--
<?php
$user = $argc + 1;
$old =& $user;
global $user, $created;
echo $old, ":", (int) isset($created), ":", (int) array_key_exists("created", $GLOBALS), "\n";
$user = 4;
$created =& $user;
unset($user);
global $user, $created;
echo (int) isset($user), ":", $created, ":", $old, "\n";
$user = 7;
$created = 9;
echo $GLOBALS["user"], ":", $GLOBALS["created"], ":", $old, "\n";
--EXPECT--
2:0:1
0:4:4
7:9:9
