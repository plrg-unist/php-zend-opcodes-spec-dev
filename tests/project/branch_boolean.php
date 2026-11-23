<?php
declare(strict_types=1);
$steps = 0;
$zero = (string) ($argc - 1);
$truth = "\0" . $argc;
$a = $zero && strlen($argc);
$b = $truth || $undefined;
$c = $truth && ++$steps;
$d = $zero || ++$steps;
$e = ($truth && $zero) || ($zero && ++$steps);
echo gettype($a), (int) $a, ":", gettype($b), (int) $b, "\n";
echo gettype($c), (int) $c, ":", gettype($d), (int) $d, ":", (int) $e, ":", $steps, "\n";
