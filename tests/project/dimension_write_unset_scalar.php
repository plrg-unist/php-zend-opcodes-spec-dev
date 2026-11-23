<?php
$a = [1 => $argc];
echo $a[1], ":";
unset($a[1][$missing]);
echo "reached";
