--TEST--
An optimized mixed array keeps its deleted slots when an exhausted cursor meets new entries
--FILE--
<?php
$a = array_intersect(
    ['a' => 1, 'b' => 2, 'c' => 3, 'd' => 4, 'e' => 5, 'f' => 6, 'g' => 7, 'h' => 8],
    [1, 3, 5, 6, 7, 8]
);
end($a);
next($a);
$a['x'] = 9;
$a['y'] = 10;
echo key($a) ?? '-', ':';
$a['z'] = 11;
echo key($a) ?? '-', '|';
$iter = [0];
foreach ($iter as $key => &$value) {
    if ($value === 0) {
        $iter = $a;
        unset($a);
    } else {
        echo $key;
    }
}
echo "\n";
--EXPECT--
-:z|z
