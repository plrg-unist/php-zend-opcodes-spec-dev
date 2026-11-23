<?php
echo $argc, ":", gettype($argc), "\n";
$other = $argc;
$argc = false;
$argv = "\0\xff";
echo $other, ":", gettype($argc), ":", gettype($argv), ":", $argv;
