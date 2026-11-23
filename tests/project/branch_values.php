<?php
declare(strict_types=1);
$steps = 0;
$zero = $argc - 1;
$bytes = "\0\xff" . $argc;
$a = $bytes ?: strlen($argc);
$b = $zero ?: ++$steps;
$c = $zero ?? ++$steps;
$d = false;
$d = $d ?? ++$steps;
$e = $missing ?? $alsoMissing ?? ++$steps;
$f = ($zero - 0.0) ?? ++$steps;
$g = "" ?? ++$steps;
echo gettype($a), "=", $a, ":", gettype($b), "=", $b, "\n";
echo gettype($c), "=", $c, ":", gettype($d), "=", (int) $d, "\n";
echo gettype($e), "=", $e, ":", gettype($f), "=", $f, ":", gettype($g), "=", strlen($g), ":", $steps, "\n";
