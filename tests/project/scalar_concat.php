<?php
$left = "\0\xff";
$right = "A\xc3\xa9";
$yes = true;
$no = false;
$nothing = null;
$number = -17;
echo $left . $right, ":", $right . $left, "\n";
echo $number . $nothing, ":", $yes . $no, ":", (string) $nothing, ":", (string) $yes, ":", (string) $number, "\n";
echo gettype((string) $number), ":", gettype($number . $nothing), "\n";
$copy = $left;
echo ($left .= $right), ":", $left, ":", $copy, "\n";
$left .= $left;
echo $left, "\n";
$string = "[$nothing$number:$yes$no|$copy$right]";
echo $string, ":", strlen($string), "\n";
echo "$no", ":", "$yes$number", ":", "$copy$right", "\n";
$written = printf("(%s)", $copy);
echo ":", $written, "\n";
