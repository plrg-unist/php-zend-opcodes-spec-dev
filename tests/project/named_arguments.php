<?php
declare(strict_types=1);

function label(int $number = 4, string $prefix = "v"): string {
    return $prefix . $number;
}

echo label(), " ", label(prefix: "n", number: 7), " ", label(prefix: "z"), "\n";
try {
    label("3");
} catch (TypeError $error) {
    echo "strict\n";
}
