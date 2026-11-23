<?php
$sentinel = "kept";
foreach ($missing as $sentinel) {
    echo "unexpected";
}
foreach ($missingWrite as &$sentinel) {
    echo "unexpected";
}
echo $sentinel, ":";
foreach ([null, false, true, 42, 1.5, "two"] as $bad) {
    echo "[";
    foreach ($bad as $v) {
        echo "unexpected";
    }
    foreach ($bad as &$v) {
        echo "unexpected";
    }
    echo "]";
}
$a = [1, 2];
foreach ($a as &$v) {
    echo "{", $v, "}";
    $a = false;
}
echo $v, "\n";
