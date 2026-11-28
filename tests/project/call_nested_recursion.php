<?php
function walk($depth, $value) {
    $saved = $value;
    if ($depth <= 0) return $value;
    return $saved + walk($depth - 1, walk($depth - 2, $value + 1));
}

function withOutput($value) {
    echo "[", $value, "]";
    return $value;
}

$local = $argc + 2;
echo walk($local, 4), ":", $local, "\n";
withOutput(withOutput($local));
echo "\n";
$words = ["zero", "one"];
echo "rope $local {$words[withOutput(1)]}/$local\n";
