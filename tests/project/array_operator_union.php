<?php
$nan = [NAN * $argc];
$left = [2 => $nan, "x" => "left", "02" => "string"];
$right = ["x" => "right", 2 => "ignored", -3 => $argc, "other" => $nan];
$joined = $left + $right;
echo (int) ($joined === [2 => $nan, "x" => "left", "02" => "string", -3 => 1, "other" => $nan]),
     (int) ($joined["other"] === $left[2]), (int) ($joined[2] === $right["other"]), "\n";
$joined[2][0] = "new";
echo $joined[2][0], (int) ($joined["other"] === $nan), (int) ($left[2] === $nan), "\n";
$past = [$argc + 10 => "gone", "keep" => 1];
unset($past[11]);
$incoming = [$argc + 30 => "gone", "keep" => 2];
unset($incoming[31]);
$merged = $past + $incoming;
$merged[] = "next";
echo (int) ($merged === ["keep" => 1, 12 => "next"]), (int) isset($merged[32]), "\n";
$empty = [];
$fresh = $empty + $nan;
echo (int) (($nan + $empty) === $nan), (int) (($nan + $nan) === $nan), (int) ($fresh === $nan), "\n";
$alias = $nan;
$result = ($nan += $alias);
echo (int) ($result === $alias), (int) ($nan === $alias), "\n";
$result = ($nan += [0 => $argc]);
echo (int) ($result === $nan), (int) ($nan === $alias), (int) ($result === $alias), "\n";
$unique = [$argc];
$unique += [0 => "ignored", 2 => $nan];
echo $unique[0], (int) ($unique[2] === $nan), "\n";
$nested = [$alias];
$copy = $nested;
$same = ($nested[0] += $alias);
$different = ($nested[0] += [0 => $argc]);
echo (int) ($same === $alias), (int) ($different === $nested[0]),
     (int) ($nested[0] === $copy[0]), (int) ($copy[0] === $alias), "\n";
