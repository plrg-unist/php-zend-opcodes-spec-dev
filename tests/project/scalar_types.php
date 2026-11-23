<?php
$null = null;
$false = false;
$true = true;
$integer = -17;
$float = -0.0;
$bytes = "\xff\0";
echo gettype($null), ":", gettype($false), ":", gettype($true), ":", gettype($integer), ":", gettype($float), ":", gettype($bytes), "\n";
echo is_null($null), ":", is_null($false), ":", is_bool($false), ":", is_bool($true), ":", is_bool($integer), "\n";
echo is_int($integer), ":", is_int($float), ":", is_float($float), ":", is_float($integer), "\n";
echo is_string($bytes), ":", is_string($null), "\n";
echo gettype(is_int($integer)), ":", gettype(is_int($float)), "\n";
echo is_scalar($null), ":", is_scalar($false), ":", is_scalar($true), ":", is_scalar($bytes), "\n";
echo $true !== false, ":", $false !== true, ":", $null !== null, ":", $integer !== null, "\n";
if (is_float($float)) { echo "float"; } else { echo "wrong"; }
if (is_int($bytes)) { echo "wrong"; } else { echo "bytes"; }
if (!is_bool($integer)) { echo "int"; } else { echo "wrong"; }
if (!is_bool($false)) { echo "wrong"; } else { echo "bool"; }
