<?php
const X = 3;
const Y = 2;
const N = null;
const UNARY_RESULTS = [+X, -X, !X, ~X, (bool) X, (int) X, (float) X, (string) X];
const ARITHMETIC = [X + Y, X - Y, X * Y, X / Y, X % Y, X << Y, X >> Y, X . Y, X | Y, X & Y, X ^ Y, X ** Y];
const COMPARE = [X === Y, X !== Y, X == Y, X != Y, X < Y, X <= Y, X > Y, X >= Y, X <=> Y, (X xor Y)];
const SELECT = [
    X && Y, X || MISSING, N && MISSING, N || Y, N ?? X, X ?? MISSING,
    N ? MISSING : Y, X ? Y : MISSING, X ?: MISSING,
];
const WRAP = (array) X;
const KEEP = (array) WRAP;
const SUM = WRAP + [1 => Y];

foreach ([UNARY_RESULTS, ARITHMETIC, COMPARE, SELECT, WRAP, KEEP, SUM] as $row) {
    foreach ($row as $v) {
        echo gettype($v), "=", $v, "|";
    }
    echo "\n";
}
