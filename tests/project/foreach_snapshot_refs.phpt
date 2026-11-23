--TEST--
A by-value foreach keeps its original keys and observes updates through shared elements
--FILE--
<?php
$a = [10, 20, 30];
$alias =& $a[1];
foreach ($a as $key => $value) {
    echo $key, "=", $value, ":";
    if ($key === 0) {
        $alias = 21;
        $a[0] = 90;
        unset($a[2]);
        $a[4] = 40;
    }
}
echo "|";
foreach ($a as $key => $value) {
    echo $key, "=", $value, ":";
}
echo "\n";

$b = [1, 2, 3];
foreach ($b as &$tail) {
    $tail *= 10;
}
foreach ($b as $tail) {
    echo $tail, ":";
}
echo "|", $b[2], "\n";

$a = [0, 1];
$before = $a;
foreach ($a as $key => &$element) {
    if ($key === 0) {
        $during = $a;
        continue;
    }
    $element = 9;
}
echo $before[1], $during[1], $a[1], "\n";
--EXPECT--
0=10:1=21:2=30:|0=90:1=21:4=40:
10:20:20:|20
199
