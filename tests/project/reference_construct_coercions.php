<?php
$a = [$missing => &$x, 2.5 => &$y, null => &$z];
$x = 3;
$y = 4;
$z = 5;
echo count($a), ":", $a[""], $a[2], ":", $x, $y, $z, "\n";
$bad = [];
echo "K";
$a = [$bad => &$x];
