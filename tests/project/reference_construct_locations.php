<?php
$source = [[$argc]];
$copy = $source;
$a = [&$source[0][0], &$source[0][1], "box" => &$source[0]];
$a[0] += 2;
$a[1] = $a[0] + 1;
$r = $a;
$r["box"][] = $a[0] + $a[1];
echo $source[0][0], $source[0][1], $source[0][2], ":", count($copy[0]), $copy[0][0], "\n";
unset($source[0]);
$r[1]++;
echo $a["box"][1], $a[1], ":", count($source), "\n";
