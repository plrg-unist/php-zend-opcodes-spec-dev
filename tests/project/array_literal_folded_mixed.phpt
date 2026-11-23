--TEST--
The optimized intersection retains mixed storage after its last string key is removed
--FILE--
<?php
$a = array_intersect([0, 1, 2, 3, "drop" => -1], [0, 1, 2, 3]);
$a[7] = 7;
unset($a[7]);
$a[4] = 8;
$a[] = 9;
echo $a[5] ?? "-", ":", $a[8] ?? "-", "\n";
--EXPECT--
-:9
