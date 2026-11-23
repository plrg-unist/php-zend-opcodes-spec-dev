<?php
$bytes = "\0\xff0" . $argc;
$copy = $bytes;
echo $bytes[0], $bytes[-4], $bytes[-1], $bytes[3], "|";
echo gettype($bytes[-5]), strlen($bytes[4]), "\n";
echo ($bytes[-4] = "\x80"), ($bytes[-1] = "XY"), "|", $bytes, "\n";
echo ($bytes[4] = 7), ($bytes[7] = true), "|", $bytes, "|", $copy, "\n";
$nested = [$bytes];
$other = $nested;
echo ($nested[0][-1] = "\xff"), "|", $nested[0], "|", $other[0], "\n";
$empty = "";
echo gettype($empty[-1] = $missing), "|", ($empty[1] = "e"), "|", $empty, "\n";
$holder = [NAN * $argc, "s"];
$copy = $holder;
echo (int) ($holder === $copy), gettype($holder[1][-2] = $missing), "|",
     (int) ($holder === $copy), $holder[1], $copy[1], "\n";
