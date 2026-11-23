<?php
$one = $argc;
$negative = -3 * $one;
$min = PHP_INT_MIN;
$max = PHP_INT_MAX;
echo gettype($negative & $one), ":", $negative & $one, ":", $negative | 6, ":", $negative ^ 6, ":", ~$negative, "\n";
echo $one << 0, ":", $one << 63, ":", $one << 64, ":", $one << $max, ":", $negative << 62, "\n";
echo $negative >> 0, ":", $negative >> 1, ":", $min >> 63, ":", $negative >> 64, ":", $negative >> $max, "\n";
echo $one >> 63, ":", $one >> 64, ":", $max >> 1, ":", $min ^ $max, "\n";
$mask = -2 * $one;
echo ($mask &= 7), ":", $mask, ":", ($mask |= 8), ":", $mask, ":", ($mask ^= 3), ":", $mask, "\n";
echo ($mask <<= 61), ":", $mask, ":", ($mask >>= 62), ":", $mask, "\n";
$float = $argc + 4.75;
$prefix = "2.5\0tail";
$infinite = $argc ? INF : 0.0;
echo $one & $prefix, ":", $float ^ 7, ":", ~$float, ":", ~$infinite, ":", $float >> $prefix, "\n";
echo $one | true, ":", false ^ $one, ":", null & $one, ":", $one << " \t+1 ", ":", $one >> null, "\n";
echo $one & "9223372036854775808", ":", $one ^ "-9223372036854775809", ":", $one | "1e400", "\n";
