--TEST--
Duplicate literal keys change the append position by reserving a wider packed array
--FILE--
<?php
$wide = [0, 1, 2, 3, 0 => 0, 0 => 0, 0 => 0, 0 => 0, 0 => 0];
$wide[12] = 7;
unset($wide[12]);
$wide[4] = 8;
$wide[] = 9;
echo $wide[5] ?? "-", ":", $wide[13] ?? "-", "\n";

$small = [0, 1, 2, 3];
$small[12] = 7;
unset($small[12]);
$small[4] = 8;
$small[] = 9;
echo $small[5] ?? "-", ":", $small[13] ?? "-", "\n";
--EXPECT--
9:-
-:9
