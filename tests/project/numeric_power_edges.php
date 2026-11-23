<?php
$one = $argc;
$zero = 0;
$negativeZero = -0.0;
echo $zero ** -$one, ":", $negativeZero ** -$one, ":", $negativeZero ** (-2 * $one), "\n";
echo (-$one) ** PHP_INT_MAX, ":", (-$one) ** (PHP_INT_MAX - 1), ":", $zero ** 0, ":", 1 ** PHP_INT_MAX, "\n";
$base = 4 * $one;
echo $base ** 32, ":", $base ** 33, ":", gettype($base ** 33), "\n";
$odd = (3 * $one) ** 40;
$square = (4096 * $one) ** 8;
echo gettype($odd), ":", $odd, "/", gettype($square), ":", $square, "\n";
$base = 3 * $one;
$exponent = 61 * $one;
$integerPower = $base ** $exponent;
$floatingPower = ((float) $base) ** $exponent;
echo gettype($integerPower), ":", ($integerPower < $floatingPower), ":", ($integerPower === $floatingPower), "\n";
echo $zero ** -0.5, ":", $negativeZero ** -0.5, "\n";
