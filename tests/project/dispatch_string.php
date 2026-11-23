<?php
for ($i = 0; $i < 8; ++$i) {
    $key = match ($i) {
        0 => "\0\xff", 1 => "Aa", 2 => "BB", 3 => "",
        4 => false, 5 => true, 6 => "\0\xfe", default => 13,
    };
    switch ($key) {
        case "\0\xff": echo "\xff"; break;
        case "Aa":
        case "BB": echo "G";
        case "": echo "Z"; break;
        default: echo "D";
    }
}
echo "\n";
