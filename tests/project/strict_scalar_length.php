<?php
declare(strict_types=1);
$bytes = "\0\xff";
$number = 17;
try {
    echo strlen($bytes), ":";
} catch (TypeError $error) {
    echo "unreachable";
}
echo strlen($number);
echo "unreachable";
