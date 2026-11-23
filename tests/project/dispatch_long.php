<?php
for ($i = 0; $i < 8; ++$i) {
    $key = match ($i) {
        0 => PHP_INT_MIN, 1 => PHP_INT_MAX, 2 => -17, 3 => 1,
        4 => "01", 5 => 1.0, 6 => 2, default => "missing",
    };
    switch ($key) {
        case PHP_INT_MIN: echo "L"; break;
        case PHP_INT_MAX: echo "H"; break;
        case -17:
        case 1: echo "G"; break;
        case 3: echo "T"; break;
        default: echo "D";
    }
}
echo "\n";
