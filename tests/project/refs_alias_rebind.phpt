--TEST--
Rebinding and unsetting aliases preserve assignment-expression values
--FILE--
<?php
$left = $argc;
$right = 8;
$x =& $left;
$snapshot = ($y =& $x);
$y++;
$x =& $right;
++$x;
unset($y);
$y = 20;
echo $left, ":", $right, ":", $snapshot, ":", $y, "\n";

$alias =& $new;
echo (int) isset($new);
$alias = 4;
echo $new;
unset($new);
$alias += 2;
echo (int) isset($new), $alias, "\n";
--EXPECT--
2:9:1:20
0406
