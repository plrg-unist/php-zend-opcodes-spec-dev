<?php
function color_group($color) {
    return match ($color) {
        "red", "orange" => "warm",
        "blue", "green" => "cool",
        "tan" => "neutral",
        default => "unknown",
    };
}

foreach (["red", "orange", "green", "blue", "tan", "white", 1] as $value) {
    echo color_group($value), "\n";
}
