<?php
$a = null;
$b = false;
$a[][] = $argc;
$result = ($b[][$argc][] += 3);
echo (int) ($a === [[1]]), (int) ($b === [[1 => [3]]]), (int) ($result === 3), ":";
$c = [false, null];
unset($c[0]["missing"], $c[1]["missing"]);
echo (int) ($c === [false, null]), ":";
$c[0][1] = 8;
$c[1][1] = 9;
echo (int) ($c === [[1 => 8], [1 => 9]]), "\n";
