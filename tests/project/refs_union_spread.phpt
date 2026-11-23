--TEST--
Array union and spread preserve references while their other owners exist
--FILE--
<?php
$v = $argc;
$a = [5 => &$v, "x" => &$v];
$spread = [...$a];
$right = ["p" => 9] + $a;
$left = $a + ["p" => 9];
$spread["x"] = 6;
echo $a[5], $spread[0], $right[5], $left["x"], $v, "\n";

$v = 3;
$source = ["v" => &$v];
unset($v, $a, $spread, $right, $left);
$spread = [...$source];
$right = ["p" => 1] + $source;
$left = $source + ["p" => 1];
$spread["v"] = 8;
$right["v"] = 9;
$left["v"] = 7;
echo $source["v"], $spread["v"], $right["v"], $left["v"], "\n";

$x = 1;
$rhs = ["drop" => 0, "keep" => &$x];
$first = ["drop" => &$x];
unset($x);
$merged = [...$first, ...($first = $rhs)];
$merged["keep"] = 9;
echo $rhs["keep"], $merged["keep"], "\n";
--EXPECT--
66666
3897
19
