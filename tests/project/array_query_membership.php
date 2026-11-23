<?php
$int = $argc;
$text = (string) $argc;
$null = $argc ? null : 1;
$bool = $argc === 1;
$float = (float) $argc;
$array = [$argc];
echo (int) in_array($int, ["1", 8], true), (int) in_array($int, ["1", 1], true),
     (int) in_array($text, [1], true), (int) in_array($text, [1, "1"], true), "\n";
echo (int) in_array($null, ["", "abc"]), (int) in_array($null, ["abc"]),
     (int) in_array(!$bool, [""]), (int) in_array($bool, ["", "abc"]),
     (int) in_array($bool, [""]), "\n";
echo (int) in_array($float, ["1"], true), (int) in_array($float, ["abc"]),
     (int) in_array($array, ["abc"]), (int) in_array($array, [1, "1"], true), "\n";
echo (int) in_array($uninitialized, ["", "abc"]), (int) in_array($uninitialized, ["", 1], true), "\n";
$bytes = "x\0" . $argc;
if (in_array($bytes, ["x\0", "x\01", "x\x001"]) && !in_array($bytes, ["x"])) {
    echo "binary\n";
}
