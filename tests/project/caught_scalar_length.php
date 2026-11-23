<?php
declare(strict_types=1);
$number = 17;
echo "T";
try {
    echo strlen($number);
} catch (TypeError $error) {
    echo "caught";
}
