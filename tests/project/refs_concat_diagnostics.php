<?php
$plain = [];
$plain .= $plain;
echo $plain, "\n";

$wrapped = [];
$alias =& $wrapped;
unset($alias);
$wrapped .= $wrapped;
echo $wrapped, "\n";
