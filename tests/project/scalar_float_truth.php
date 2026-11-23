<?php
$positiveZero = 0.0;
$negativeZero = -0.0;
$positiveTiny = 5e-324;
$negativeTiny = -5e-324;
$regular = -2.25;
$infinity = INF;
$negativeInfinity = -INF;
$nan = NAN;
$negativeNan = -NAN;
echo (bool) $positiveZero, ":", (bool) $negativeZero, ":", (bool) $positiveTiny, ":", (bool) $negativeTiny, "\n";
echo !$regular, ":", !$infinity, ":", !$negativeInfinity, "\n";
if ($negativeZero) { echo "wrong"; } else { echo "zero"; }
if ($negativeTiny) { echo "tiny"; } else { echo "wrong"; }
echo ":", (bool) $nan, ":", !$nan, "\n";
echo ($nan xor $nan), "\n";
if ($nan) { echo "nan"; } else { echo "wrong"; }
echo ":", (bool) $negativeNan, "\n";
return $negativeZero;
