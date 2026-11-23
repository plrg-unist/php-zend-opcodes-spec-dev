<?php
$zero = $argc - 1;
echo "C";
try {
    echo $argc / $zero;
} catch (DivisionByZeroError $error) {
    echo "caught";
}
