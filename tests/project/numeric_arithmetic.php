<?php
$one = $argc;
$negative = -3 * $one;
$seven = 7 * $one;
echo $seven + $negative, ":", $negative - $seven, ":", $seven * $negative, "\n";
$exact = $negative / $negative;
$fraction = $seven / $negative;
echo gettype($exact), ":", $exact, "/", gettype($fraction), ":", $fraction, "\n";
echo $negative % $seven, ":", $seven % $negative, ":", PHP_INT_MIN % -$one, "\n";

$max = PHP_INT_MAX;
$min = PHP_INT_MIN;
$high = $max + $one;
$low = $min - $one;
$product = $max * ($one + 1);
$quotient = $min / -$one;
echo gettype($high), ":", $high, ":", $high === 9223372036854775808.0, "\n";
echo gettype($low), ":", $low, ":", $low === -9223372036854775808.0, "\n";
echo gettype($product), ":", $product === 18446744073709551616.0, "\n";
echo gettype($quotient), ":", $quotient === $high, "\n";
$rounded = $max + (1025 * $one);
echo $rounded - (float) $max, "\n";
$round = 9007199254740992.0 + $one;
$next = $round + 2;
echo ($round === 9007199254740992.0), ":", ($next === 9007199254740994.0), "\n";
$multiple = 0.5 * $seven;
echo gettype($multiple), ":", $multiple, ":", $multiple - $one, ":", $one / $multiple, "\n";

$power = (-3 * $one) ** 5;
$overflow = ($one + 1) ** 63;
echo gettype($power), ":", $power, ":", gettype($overflow), ":", $overflow === $high, "\n";
$big = 16 * $one;
echo $big ** 0, ":", $big ** -2, ":", $big ** 0.5, ":", (-$one) ** 0.5, "\n";
$round *= 2;
echo $round === 18014398509481984.0, "\n";
$x = $seven;
echo ($x += $negative), ":", ($x -= $negative), ":", ($x *= -2), ":", ($x /= 2), ":";
echo ($x %= 3), ":", ($x **= 4), ":", gettype($x), ":", $x, "\n";
