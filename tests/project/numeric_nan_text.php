<?php
$n = NAN;
echo $n, ":", (string) $n, ":", strlen($n), "\n";
echo $n . $missing, ":", "$n$missing", ":", "[$missing:$n:$missing]", "\n";
$left = $n;
echo ($left .= $missing), ":", $left, "\n";
$right = "[";
echo ($right .= $n), ":", $right, "\n";
