<?php
$null = $argc ? null : "x";
$false = $argc !== 1;
$empty = $argc ? [] : ["x"];
$full = [$argc];
$nan = $argc ? NAN : 0.0;
echo (int) isset($absent), (int) empty($absent), "|",
     (int) isset($null), (int) empty($null), "|",
     (int) isset($false), (int) empty($false), "\n";
echo (int) isset($empty), (int) empty($empty), "|",
     (int) isset($full), (int) empty($full), "|",
     (int) isset($nan), (int) empty($nan), "\n";
if (isset($full) && !empty($full)) {
    echo "full\n";
}
if (isset($absent) || empty($empty)) {
    echo "empty\n";
}
