<?php
$suffix = $argc === 1 ? "" : "x";
$a = [
    "0" . $suffix => "zero", "-0" . $suffix => "negative zero",
    "+1" . $suffix => "plus", "01" . $suffix => "leading zero",
    " 1" . $suffix => "space", "1\0" . $suffix => "binary",
    "-9223372036854775808" . $suffix => "minimum", "9223372036854775807" . $suffix => "maximum",
    "-9223372036854775809" . $suffix => "below", "9223372036854775808" . $suffix => "above",
    "1.0" . $suffix => "dot", "1e0" . $suffix => "exponent", "1" . $suffix => "last",
];
$expected = [
    0 => "zero", "-0" => "negative zero", "+1" => "plus", "01" => "leading zero",
    " 1" => "space", "1\0" => "binary", -9223372036854775807 - 1 => "minimum",
    9223372036854775807 => "maximum", "-9223372036854775809" => "below",
    "9223372036854775808" => "above", "1.0" => "dot", "1e0" => "exponent", 1 => "last",
];
echo (int) ($a === $expected), "/", count($a), "/", $a[0], "/", $a[1], "/", $a["1\0"], "\n";
