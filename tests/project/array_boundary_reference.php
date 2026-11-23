<?php
$value = $argc;
echo "ref";
$a = [&$value];
echo $a[0];
