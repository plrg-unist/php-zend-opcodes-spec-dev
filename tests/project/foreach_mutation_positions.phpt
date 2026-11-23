--TEST--
Mixed compaction and deleting the packed tail relocate active foreach positions
--FILE--
<?php
$a = ["a" => 1, "b" => 2, "c" => 3, "d" => 4,
      "e" => 5, "f" => 6, "g" => 7, "h" => 8];
foreach ($a as $k => &$v) {
    echo $k, $v, ":";
    if ($k === "c") {
        unset($a["a"], $a["b"], $a["d"]);
        $a["i"] = 9;
    }
    if ($k === "i") {
        unset($a["g"], $a["h"], $a["i"]);
        $a["j"] = 10;
    }
}
unset($v);
echo "|";
$b = [1, 2, 3];
foreach ($b as $k => &$v) {
    echo $k, $v, ":";
    if ($v === 2) {
        unset($b[1], $b[2]);
        $b[1] = 7;
        $b[2] = 8;
    }
}
echo "\n";
--EXPECT--
a1:b2:c3:e5:f6:g7:h8:i9:j10:|01:12:17:28:
