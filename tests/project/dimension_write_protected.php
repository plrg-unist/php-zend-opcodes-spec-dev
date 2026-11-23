<?php
$a = [];
$a[$argc] = true;
echo (int) $a[$argc], ":";
try {
    $a[$argc]["nested"] = 20;
} catch (Error $failure) {
    echo "caught";
}
