<?php
$empty = "";
$bytes = "a\0\xff\xc3\xa9";
$false = false;
$true = true;
$min = PHP_INT_MIN;
$zero = 0;
$null = null;
echo strlen($empty), ":", strlen($bytes), ":", strlen($false), ":", strlen($true), "\n";
echo strlen($min), ":", strlen($zero), "\n";
echo gettype(strlen($bytes)), "\n";
echo "null=", strlen($null), "\n";
