<?php
function leaf($a) {
    $a = 7;
    return 1 / 0;
}
function wrapper(#[SensitiveParameter] $secret, $value) {
    unset($value);
    return leaf($secret);
}
function accept($first, $second) { return $first; }

echo "before\n";
accept(10, wrapper("hidden", 2, "A\n\r\t\f\v\e\\'\x00\x1f\x7f\xff012345678901234",
    "123456789012345", "1234567890123456", true, false, null, -2, -0.0, 9.5, NAN, [], STDIN));
