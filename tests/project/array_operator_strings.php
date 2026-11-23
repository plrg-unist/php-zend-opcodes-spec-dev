<?php
$v = [NAN * $argc];
$a = $v;
$b = $v;
echo "self[", ($a .= $a), "]", $a, "\n";
echo "alias[", ($b .= $v), "]", $b, ":", gettype($v), "\n";
echo "both[", $v . $v, "]\n";
echo "missing[", $v . $absent, "]\n";
echo "leading[", $other . $v, "]\n";
echo "fast[", "{$v}{$absent}", "]\n";
echo "rope[", "{$v}|{$missing}|{$v}", "]\n";
$float = NAN * $argc;
echo "different[", $v . $float, "|", $float . $v, "]\n";
echo "same float[", ($float .= $float), "]", $float, "\n";
$nested = [$v];
$copy = $nested;
echo "nested[", ($nested[0] .= $v), "]", $nested[0], (int) ($copy[0] === $v), "\n";
$text = "a\0c";
echo "offset[", ($text[1] = $v), "]", $text, "\n";
