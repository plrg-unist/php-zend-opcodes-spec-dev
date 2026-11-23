--TEST--
A dynamically created variable named GLOBALS is an ordinary entry in the global snapshot
--FILE--
<?php
$name = 'GLOBALS';
echo (int) isset($$name), ':', (int) isset($GLOBALS['GLOBALS']), "\n";
$$name = [3, 4];
echo $$name[1], ':', $GLOBALS['GLOBALS'][0], "\n";
$snapshot = $GLOBALS;
$snapshot['GLOBALS'][1] = 9;
echo $$name[1], ':', $snapshot['GLOBALS'][1], "\n";
unset($$name);
echo (int) isset($GLOBALS['GLOBALS']), ':', count($snapshot['GLOBALS']), "\n";
--EXPECT--
0:0
4:3
4:9
0:2
