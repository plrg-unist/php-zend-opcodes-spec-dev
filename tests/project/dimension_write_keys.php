<?php
$key = $argc;
$a = [8 => "first", 3 => "second"];
unset($a[8], $a[3]);
$value = ($a[] += $key);
echo (int) ($value === 1), (int) ($a === [9 => 1]), ":";
$a[true] = 10;
$a[1.5] += 2;
$a["01"] = 3;
$a["-1"] = 6;
$a[null] = "empty";
unset($a[false], $a[null], $a[-1.5]);
echo (int) ($a === [9 => 1, 1 => 12, "01" => 3]), "\n";
$max = [PHP_INT_MAX => 3];
unset($max[PHP_INT_MAX]);
$max[] = 13;
echo (int) ($max === [PHP_INT_MAX => 13]), "\n";
