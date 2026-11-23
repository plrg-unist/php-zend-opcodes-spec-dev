<?php
$a = [PHP_INT_MAX => $argc];
echo $a[PHP_INT_MAX], "\n";
$a[] += $undefined;
echo "reached";
