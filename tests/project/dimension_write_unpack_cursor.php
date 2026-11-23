<?php
$rhs = [$argc, $argc];
$packed = [7 => $argc, ...$rhs];
unset($packed[9], $packed[8]);
$packed[8] = 0;
$packed[] = 42;
echo (int) isset($packed[9]), (int) isset($packed[10]), ":";
$rhs["temp"] = 0;
unset($rhs["temp"]);
$mixed = [7 => $argc, ...$rhs];
unset($mixed[9], $mixed[8]);
$mixed[8] = 0;
$mixed[] = 42;
echo (int) isset($mixed[9]), (int) isset($mixed[10]), "\n";
$mapped = ["x" => $argc, ...[$argc + 1]];
$mapped[] = "end";
echo (int) ($mapped === ["x" => 1, 0 => 2, 1 => "end"]), "\n";
