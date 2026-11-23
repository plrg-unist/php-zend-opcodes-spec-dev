--TEST--
Unsetting an array releases its element aliases unless a cycle keeps them alive
--FILE--
<?php
$x = 1;
$a = [&$x];
$g = [&$x];
unset($g, $x);
$b = $a;
$b[0] = 7;
echo $a[0], $b[0], "\n";

$x = 1;
$a = [&$x];
$g = [];
$g[0] = &$g;
$g[1] = &$x;
unset($g, $x);
$b = $a;
$b[0] = 7;
echo $a[0], $b[0], "\n";
--EXPECT--
17
77
