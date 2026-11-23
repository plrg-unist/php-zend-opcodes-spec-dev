<?php
namespace Iteration;
$path = PHP_BINARY;
$last = "kept";
foreach ($path as $last) { echo "unexpected"; }
foreach ($path as &$last) { echo "unexpected"; }
echo $last, "\n";

$live = [1, 2];
foreach ($live as &$value) {
    echo $value;
    $live = $path;
}
echo ":", $value, "\n";

[$missing => $left, $path => $right] = $path;
echo (int) ($left === null), (int) ($right === null), "\n";
