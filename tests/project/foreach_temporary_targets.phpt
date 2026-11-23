--TEST--
Foreach temporaries and reference or value assignment to dimensions preserve the selected element
--FILE--
<?php
$n = 4;
foreach ([2, &$n] as &$v) {
    $v++;
    echo $v, ":";
}
$v = 7;
echo "|", $n, ":";
unset($v);
foreach ([1, 2] as &$v) {
    echo $v;
}
$v = 8;
echo ":", $v, "\n";

$b = [7, 9];
$out = [];
foreach ($b as $out[]) {
}
echo $out[0], $out[1], ":";
foreach ($b as &$out[0]) {
    $out[0]++;
}
$out[0] = 6;
echo $b[0], $b[1], ":", $out[0], $out[1], "\n";
--EXPECT--
3:5:|7:12:8
79:86:69
