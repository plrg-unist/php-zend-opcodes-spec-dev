<?php
$low = PHP_INT_MIN;
$high = PHP_INT_MAX;
echo $low, "\x00", $high, "\xff";
$near = 9223372036854775806;
echo 0, ":", ++$near, "\n";
$x = -11;
echo ($y = $x), ":", ++$x, ":", $y, ":", $x, "\n";
echo ($y = ++$x), ":", $y, ":", $x, "\n";
$x = $x;
$less = $low < $high;
$equal = $x < $y;
if ($less) { echo "less"; }
if ($equal) { echo "wrong"; } else { echo "equal"; }
if ($high < $low) { echo "wrong"; } else { echo "greater"; }
$empty = "";
echo $empty, "\n";
$nothing = null;
return $nothing;
