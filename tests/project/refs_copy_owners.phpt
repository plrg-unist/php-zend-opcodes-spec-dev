--TEST--
Array copies preserve multiply owned references and separate singly owned elements
--FILE--
<?php
$value = $argc;
$a = [&$value, ["q" => 2]];
$b = $a;
$b[1]["q"] = 3;
$b[0] = 7;
echo $a[0], $a[1]["q"], $b[0], $b[1]["q"], $value, "\n";
unset($value, $b);
$b = $a;
$b[0] = 9;
echo $a[0], $b[0], "\n";

$v = 4;
$pair = [&$v, &$v];
unset($v);
$other = $pair;
$other[0]++;
echo $pair[0], $pair[1], $other[0], $other[1], "\n";
--EXPECT--
72737
79
5555
