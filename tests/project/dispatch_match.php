<?php
$misses = 0;
for ($i = 0; $i < 10; ++$i) {
    $key = match ($i) {
        0 => 1, 1 => "1", 2 => "01", 3 => "\0\xff", 4 => "",
        5 => true, 6 => 1.0, 7 => null, 8 => PHP_INT_MIN, default => -13,
    };
    $value = match ($key) {
        1 => 7,
        "1" => "s",
        "01" => false,
        "\0\xff", PHP_INT_MIN => "\xfe",
        "" => null,
        "trap" => $never / 0,
        default => ++$misses,
    };
    echo gettype($value), ":", $value, ";";
}
echo $misses, "\n";
