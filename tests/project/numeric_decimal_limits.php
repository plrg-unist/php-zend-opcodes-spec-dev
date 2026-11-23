<?php
$bare = "-9223372036854775808";
$space = "-9223372036854775808 ";
$nul = "-9223372036854775808\0junk";
$exp = "1999999999999999999e";
$signedExp = "1999999999999999999e+";
$overflowExp = "9300000000000000000e+";
$a = $bare + 0;
$b = $space + 0;
$c = $nul + 0;
$d = $exp + 0;
$e = $signedExp + 0;
$f = $overflowExp + 0;
echo gettype($a), ":", $a, "/", gettype($b), ":", $b, "/", gettype($c), ":", $c, "\n";
echo gettype($d), ":", $d, "/", gettype($e), ":", $e, "/", gettype($f), ":", $f, "\n";
$small = "10000000000000000000e-99";
echo ($small <=> "0"), ":", ("0" <=> $small), ":", ($small == 1e-80), "\n";
