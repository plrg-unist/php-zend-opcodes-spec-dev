<?php
function chooseFirst($first, $second) { return $first[0]; }
function replaceGlobal() { global $table; $table[0] = 2; return 0; }
function takeHeld() { global $held; $tmp = $held; $held = null; return $tmp; }
function modifyCopy($copy) { $copy[0] = 7; return $copy[0]; }
function combine($first, $second) { return $first[0] + $second; }
function combineScalars($first, $second) { return $first * 10 + $second; }

$table = [1];
echo chooseFirst($table, replaceGlobal()), ":", $table[0], "\n";
$x = 1;
$alias = &$x;
echo combineScalars($alias, ($x = 2)), "\n";
unset($alias);

$x = 1;
$first = [&$x];
$held = [&$x];
unset($x);
echo combine(takeHeld(), modifyCopy($first)), ":", $first[0], "\n";

$x = 2;
$first = [&$x];
$held = [&$x];
unset($x);
echo modifyCopy($first, takeHeld()), ":", $first[0], "\n";
