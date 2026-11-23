<?php
$a = $argc;
echo $a, ":";
unset($a[0][1]);
echo "reached";
