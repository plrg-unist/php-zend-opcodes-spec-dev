<?php
$one = $argc;
$null = $one ? null : 5;
$a = [$missingKey => $missingValue, $null => "null", $one + 1.75 => "fraction",
      $one - 1.0 => "exact", $one > 0 => "true", $one < 0 => "false"];
echo (int) ($a === ["" => "null", 2 => "fraction", 0 => "false", 1 => "true"]),
     "/", $a[""], "/", $a[2], "/", $a[0], "/", $a[1], "\n";
$b = [INF * $one => "infinity", NAN * $one => "nan", -INF * $one => "last"];
echo count($b), "/", $b[0], "\n";
