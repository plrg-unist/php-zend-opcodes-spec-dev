<?php
function caller($values, $name) {
    $value = 5;
    return Callee($values[0], $$name, callee(1, 2, 3));
}
function callee($a, $b, $c) { return $a * 100 + $b * 10 + $c; }

echo caller([$argc], 'value'), "\n";
