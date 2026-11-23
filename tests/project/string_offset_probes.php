<?php
$s = "0\0\xff" . $argc;
echo (int) isset($s[0]), (int) empty($s[0]), (int) isset($s[1]), (int) empty($s[1]),
     (int) empty($s[-2]), (int) empty($s[-4]), (int) isset($s[-5]), (int) empty($s[4]), "\n";
echo (int) isset($s[" +01\t"]), (int) empty($s["1bad"]), (int) isset($s["1bad"]),
     (int) isset($s[[]]), (int) empty($s[[]]), (int) isset($s["1.0"]),
     (int) empty($s["9223372036854775808"]), "\n";
echo (int) isset($s[-2.5]), (int) empty($s[0.25]), (int) isset($s[2.0]),
     (int) empty($s[true]), (int) empty($s[null]), (int) isset($s[$absent]), "\n";
echo (int) isset($s[NAN]), (int) empty($s[INF]), (int) isset($s[1e20]), "\n";
if (isset($s[-1]) && empty($s[0]) && !empty($s[1])) {
    echo "present\n";
}
