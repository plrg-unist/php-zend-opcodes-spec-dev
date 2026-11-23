<?php
$n = $argc;
$key = "\0\xff";
$a = [$key => [$n, "02"], 7 => null];
$b = [7 => null, $key => [(string) $n, 2]];
$c = [7 => null, $key => [$n, "02"]];
$d = [$key => [$n, "02"], 7 => null];
echo (int) ($a == $b), (int) ($a === $b), (int) ($a === $c), (int) ($a === $d), "\n";
echo $a <=> $b, ":", $b <=> $a, "\n";

$left = ["first" => $n, "left" => 0];
$right = ["first" => $n, "right" => 0];
echo $left <=> $right, ":", $right <=> $left, ":";
echo (int) ($left < $right), (int) ($left <= $right), (int) ($left > $right), (int) ($left >= $right), "\n";
$left = ["first" => -$n, "left" => 0];
$right = ["right" => 0, "first" => $n];
echo $left <=> $right, ":", $right <=> $left, ":", $right <=> [$n], "\n";
$left = ["a" => $n, "b" => 3];
$right = ["b" => $n, "a" => 3];
echo $left <=> $right, ":", $right <=> $left, "\n";

$duplicate = [$n => "old", "01" => 4, "1" => "new"];
$expected = [1 => "new", "01" => 4];
echo (int) ($duplicate == $expected), (int) ($duplicate === $expected), "\n";
