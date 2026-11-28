<?php
function leaf($value) { echo "leaf|"; return 1 / $value; }
function outer($value) { echo "outer|"; return leaf($value); }
echo "main|";
try {
    outer(0);
} catch (DivisionByZeroError $error) {
    echo "caught\n";
}
