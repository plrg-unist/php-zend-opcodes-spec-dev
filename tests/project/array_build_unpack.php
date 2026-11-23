<?php
$n = $argc;
$left = [$n + 8 => "nine", "x" => "first", "stay" => "K", $n - 4 => "minus"];
$right = ["x" => "second", $n + 80 => [$n], "z" => "Z"];
$out = [$n + 2 => "head", ...$left, "y" => "Y", ...$right, ...[], "tail"];
$expected = [3 => "head", 4 => "nine", "x" => "second", "stay" => "K", 5 => "minus",
             "y" => "Y", 6 => [1], "z" => "Z", 7 => "tail"];
echo (int) ($out === $expected), "/", $out["x"], "/", $out[6][0], "/", $out[7], "\n";
$packed = [$n, $n + 1];
$sparse = [2 => "a", 6 => "b"];
$joined = [...[], ...$packed, ...$sparse, ...$packed];
echo (int) ($joined === [1, 2, "a", "b", 1, 2]), "/", count($joined), "\n";
