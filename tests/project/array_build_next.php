<?php
$n = $argc;
$a = [$n - 6 => "a", "b", $n - 20 => "c", "d", $n - 2 => "e", "f"];
$expected = [-5 => "a", -4 => "b", -19 => "c", -3 => "d", -1 => "e", 0 => "f"];
echo (int) ($a === $expected), "/", $a[-4], "/", $a[-3], "/", $a[0], "\n";
$max = 9223372036854775807;
$b = [$max - $n => "old", $max - $n => "replaced", "last"];
echo (int) ($b === [9223372036854775806 => "replaced", 9223372036854775807 => "last"]),
     "/", $b[$max - $n], "/", $b[$max], "\n";
$min = -9223372036854775807 - $n;
$c = [$min => "first", "next"];
echo count($c), "/", $c[$min], "/", $c[$min + 1], "\n";
