--TEST--
Foreach destructuring aliases one element and copies another from each separated row
--FILE--
<?php
$rows = [["x" => 1, "y" => 5], ["x" => 2, "y" => 8]];
$old = $rows;
foreach ($rows as $k => ["x" => &$v, "y" => $y]) {
    $v += $y;
    $y = 4;
    echo $k, $v, ":";
}
$v++;
echo "|", $rows[0]["x"], $rows[1]["x"], ":", $rows[0]["y"], $rows[1]["y"],
    ":", $old[0]["x"], $old[1]["x"], "\n";
--EXPECT--
06:110:|611:58:12
