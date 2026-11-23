<?php
$key = $argc;
$a = [$key => [["keep" => false, "nil" => null]]];
$i = 0;
$v = ($a[$key][$i++]["keep"] ??= ++$side);
echo $i, (int) ($v === false), (int) isset($side), ":";
$i = 0;
$w = ($a[$key][$i++]["nil"] ??= "set");
$x = ($a[$key][2]["new"] ??= $key);
$y = ($a[$key][2]["new"] ??= $missing);
$plain = "present";
$a["cv"] = $plain;
unset($plain);
$plain ??= $a["cv"];
echo $i, ":", $w, ":", $x, ":", $y, ":", $plain, "\n";
echo (int) ($a[$key] === [0 => ["keep" => false, "nil" => "set"], 2 => ["new" => 1]]), "\n";
