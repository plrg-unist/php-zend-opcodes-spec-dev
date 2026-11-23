<?php
$v = $argc;
$wide = [$v, 1, 2, 3, 0 => $v, 0 => $v, 0 => $v, 0 => $v, 0 => $v];
$wide[12] = 7;
unset($wide[12]);
$wide[4] = 8;
$wide[] = 9;
echo (int) isset($wide[5]), (int) isset($wide[13]), ":";
$small = [$v, 1, 2, 3];
$small[12] = 7;
unset($small[12]);
$small[4] = 8;
$small[] = 9;
echo (int) isset($small[5]), (int) isset($small[13]), "\n";
$grown = [$v];
for ($n = 1; $n <= 8; ++$n) {
    $grown[] = $n;
}
$grown[12] = 7;
unset($grown[12]);
$grown[9] = 8;
$grown[] = 9;
echo count($grown), ":", (int) isset($grown[10]), (int) isset($grown[13]), "\n";
