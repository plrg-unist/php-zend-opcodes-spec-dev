<?php
$a = [];
$a[
    $key
] =
    $rhs;
echo (int) ($a === ["" => null]), ":";
unset($a[$missingUnsetKey], $undefined, $absent["x"]["y"]);
echo (int) ($a === []), (int) isset($absent), "\n";
$value = ($root["x"][2] += $undefinedRhs);
echo (int) ($value === 0), (int) ($root === ["x" => [2 => 0]]), "\n";
$falseValue = false;
unset($falseValue[$anotherKey]);
echo (int) ($falseValue === false), ":";
$assigned = ($falseValue["yes"] = 4);
echo $assigned, ":", $falseValue["yes"], "\n";
