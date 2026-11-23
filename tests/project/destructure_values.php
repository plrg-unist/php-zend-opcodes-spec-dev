<?php
$row = $argc === 1 ? "row" : "other";
$source = [$row => ["bytes" => "\0\xff", "numbers" => [$argc, 22, 33]], 1 => "one", "01" => "padded"];
[$row => ["numbers" => [$first, , $third], "bytes" => $bytes], "1" => $one, "01" => $padded] = $source;
echo $first, "|", $third, "|", $bytes, "|", $one, "|", $padded, "\n";

$key = 0;
[0 => $key, $key => $value] = ["next", "next" => 71, 1 => 52];
echo $key, "|", $value, "\n";

$index = $argc - 1;
$target = [];
[$index, $target[$index], $target[], $index] = [5, "at-five", "at-six", 9];
echo $index, "|", $target[5], "|", $target[6], "|", count($target), "\n";
