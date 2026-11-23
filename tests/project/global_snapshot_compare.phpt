--TEST--
Global snapshots regain equality after removing the extra global and retain shared references
--FILE--
<?php
$x = 1;
$r =& $x;
$a = $GLOBALS;
$b = $GLOBALS;
echo (int)($a == $b), (int)($a === $b), "|";
unset($b['a']);
echo (int)($a == $b), (int)($a === $b), "|";
$x = 7;
echo $a["r"], ":", $b["x"], "|";
$b["r"] = 4;
echo $a["x"], ":", $x, "\n";
--EXPECT--
00|11|7:7|4:4
