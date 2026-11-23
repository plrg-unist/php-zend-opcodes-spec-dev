<?php
$s = "AB0" . $argc;
echo $s[" +01\t"], $s[" -1 "], $s["0bad"], $s["9\0tail"], "\n";
echo $s[$absent], $s[null], $s[false], $s[true], $s[-1.75], "\n";
echo $s[NAN], $s[INF], gettype($s[PHP_INT_MIN]), gettype($s[PHP_INT_MAX]), "\n";
echo $s["1bad"] ?? "?", $s["1.0"] ?? "?", $s["1e2"] ?? "?",
     $s["9223372036854775808"] ?? "?", $s["NaN"] ?? "?", "\n";
echo $s[$absent] ?? "?", $s[true] ?? "?", $s[3.5] ?? "?", $s[-5] ?? "?", "\n";
echo ($s["1junk"] = ["xy"]), "|", $s, "\n";
echo gettype($s["-9junk"] = $absent), "|", $s, "\n";
echo ($s[$absent] = 21), ($s[2.5] = "Z"), "|", $s, "\n";
