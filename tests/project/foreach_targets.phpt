--TEST--
Foreach key and value targets can alias the same slot or a dimension
--FILE--
<?php
$a = ["x" => 6, "y" => 7];
foreach ($a as $both => &$both) {
    echo $both;
}
echo "|", $a["x"], $a["y"], ":";
unset($both);
$b = [2, 3];
foreach ($b as &$b) {
    $b += 10;
    echo $b, ":";
}
echo "|", $b, "\n";

$matrix = ["row" => [1, 2]];
$copy = $matrix;
foreach ($matrix["row"] as $index => &$v) {
    $v += 4;
    if ($index === 0) {
        unset($matrix["row"]);
    }
    echo $index, $v, ":";
}
$v = 7;
echo "|", $copy["row"][0], $copy["row"][1], ":", (int)isset($matrix["row"]), "\n";
--EXPECT--
xy|xy:12:13:|13
05:16:|12:0
