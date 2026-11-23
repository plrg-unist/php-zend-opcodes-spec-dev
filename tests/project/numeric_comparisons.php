<?php
$one = $argc;
$two = $one + 1;
$float = 2.0;
$string = "2";
$binary = "2\0\xff";
echo ($two == $float), ":", ($two === $float), ":", ($two != $float), ":", ($two !== $float), "\n";
echo ($two == $string), ":", ($two === $string), ":", ($two < $binary), ":", ($binary <=> $two), "\n";
echo ($float == $string), ":", ($float < $binary), ":", ($binary > $float), ":", ($float <=> $binary), "\n";
echo ($one <= $two), ":", ($two <= $one), ":", ($two <=> $two), ":", gettype($two == $float), ":", gettype($two <=> $one), "\n";
$nan = NAN;
$inf = INF;
$nil = null;
$true = true;
$false = false;
echo ($nan == $nan), ":", ($nan === $nan), ":", ($nan != $nan), ":", ($nan <=> $nan), "\n";
echo ($nan < $float), ":", ($float <= $nan), ":", ($nan <=> $two), ":", ($two <=> $nan), "\n";
echo ($nan <=> $string), ":", ($string <=> $nan), ":", ($nan <=> $true), ":", ($false <=> $nan), ":", ($nan <=> $nil), "\n";
echo ($inf == "INF"), ":", ($nil <=> ""), ":", ("0" <=> $nil), ":", ($nil <=> -2), ":", ($true <=> $nil), "\n";
echo ($true <=> $false), ":", ($false <=> $true), ":", ($false <=> 0), ":", ($true <=> "0"), "\n";
if ($one != $two) { echo "different"; } else { echo "wrong"; }
if ($two === $float) { echo "wrong"; } else { echo "/typed"; }
if ($float >= $two) { echo "/equal"; }
echo "\n";
