<?php
$nan = NAN;
$a = $nan && $argc;
$b = $nan || $missing;
$c = $nan ?: $missing;
$d = $nan ?? $missing;
echo gettype($a), (int) $a, ":", gettype($b), (int) $b, ":", $c, ":", $d, "\n";
$e = $missing && $skipped;
$f = $missing || $argc;
$g = $missing ?: 7;
$h = $missing ?? 8;
echo (int) $e, ":", (int) $f, ":", $g, ":", $h, "\n";
