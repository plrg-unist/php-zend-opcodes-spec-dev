--TEST--
A live foreach follows a separated branch and resets on a later rebind to an abandoned branch
--FILE--
<?php
$a = ["a" => 1, "b" => 2, "c" => 3, "d" => 4];
$step = 0;
foreach ($a as $key => &$value) {
    echo $key, $value, ":";
    if ($step === 0) {
        $left = $a;
        unset($left["b"]);
        $right = $left;
        $right["b"] = 20;
        unset($right["c"]);
        $a = $right;
    } elseif ($step === 1) {
        $a = $left;
    }
    $step++;
}
$value = 40;
echo "|", $a["d"], ":", $left["d"], ":", $right["d"], "\n";
--EXPECT--
a1:d4:a1:c3:d4:|40:4:4
