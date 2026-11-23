--TEST--
Nested foreach iterators and multi-level continue survive COW and early cleanup
--FILE--
<?php
$a = [1, 2, 3];
foreach ($a as $outer => &$left) {
    echo "[", $outer, "=";
    foreach ($a as $inner => &$right) {
        echo $inner, $right, ":";
        if ($outer === 0 && $inner === 0) {
            $snapshot = $a;
            unset($a[1]);
            $a[] = 4;
            $right = 5;
            continue 2;
        }
        if ($outer === 2 && $inner === 0) {
            $left = 6;
            break;
        }
    }
    echo "]";
}
echo "|", $snapshot[0], $snapshot[1], $snapshot[2], ":", $a[0], $a[2], $a[3], "\n";
--EXPECT--
[0=01:[2=05:][3=05:26:34:]|523:564
