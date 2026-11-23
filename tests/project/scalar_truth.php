<?php
$null = null;
$false = false;
$true = true;
$zero = 0;
$negative = -1;
$empty = "";
$zeroString = "0";
$twoZeros = "00";
$nul = "\0";
$binary = "0\xff";
echo (bool) $null, ":", (bool) $false, ":", (bool) $true, "\n";
echo (bool) $zero, ":", (bool) $negative, "\n";
echo (bool) $empty, ":", (bool) $zeroString, ":", (bool) $twoZeros, ":", (bool) $nul, ":", (bool) $binary, "\n";
echo !$empty, ":", !$zeroString, ":", !$twoZeros, ":", !$nul, "\n";
echo ($zeroString xor $true), ":", ($negative xor $true), ":", ($null xor $empty), ":", ($nul xor $false), "\n";
echo gettype((bool) $negative), ":", gettype(!$empty), ":", gettype(($false xor $true)), "\n";
if ($zeroString) { echo "wrong"; } else { echo "zero"; }
if ($twoZeros) { echo "two"; } else { echo "wrong"; }
if ($nul) { echo "\0"; } else { echo "wrong"; }
echo $zeroString ? "wrong" : $binary, ":", $nul ? $twoZeros : "wrong";
