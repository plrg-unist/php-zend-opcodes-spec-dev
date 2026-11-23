<?php
$key = $argc;
$a = ["old" => 9];
$value = ($a[$key] = [10]);
$nested = ($a[$key][] = false);
$deep = ($a["tree"][]["leaf"] = $key + 1);
$a["old"] = $a[$key][0];
echo (int) ($value === [10]), (int) ($nested === false), (int) ($deep === 2), ":";
echo (int) ($a === ["old" => 10, 1 => [10, false], "tree" => [["leaf" => 2]]]), "\n";
$a[(string) $key] = null;
$a[] = "next";
unset($a["old"], $a[$key], $a[2], $a[101]);
$a[] = "after";
echo (int) ($a === ["tree" => [["leaf" => 2]], 3 => "after"]), "\n";
unset($neverSet, $value);
echo (int) isset($value), (int) isset($neverSet), ":", $a[3], "\n";
