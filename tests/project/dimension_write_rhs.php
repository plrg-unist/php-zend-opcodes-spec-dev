<?php
$key = $argc;
$a = [$key => ["original" => 4]];
$saved = $a;
$result = ($a[$key]["slot"] = ($a[$key] = ["new" => $key]));
echo (int) ($result === ["new" => 1]), (int) ($saved === [1 => ["original" => 4]]), ":";
echo (int) ($a === [1 => ["new" => 1, "slot" => ["new" => 1]]]), "\n";
$a = [];
$a[] = $a;
$a[$key] = $a;
echo (int) ($a === [[], [[]]]), "\n";
$a[][$key][] = "late";
echo $a[2][$key][0], "\n";
