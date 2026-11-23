<?php
$key = $argc;
$original = [NAN * $key, "v" => 1];
$same = $original;
$same["v"] ??= 4;
echo (int) ($same === $original), ":";
$same["v"] = 1;
echo (int) ($same === $original), (int) ($same["v"] === $original["v"]), "\n";
$unset = $original;
unset($unset["missing"]);
echo (int) ($unset === $original), ":";
$outer = [$original, null];
$copy = $outer;
unset($copy[0]["missing"], $copy[1]["missing"], $copy[2]["missing"]);
echo (int) ($copy[0] === $outer[0]), (int) ($outer[0] === $original), (int) ($copy[1] === null), count($copy), "\n";
