<?php
function measure($mode) {
    echo "measure|";
    if ($mode === "empty") return strlen();
    if ($mode === "surplus") return strlen("a", 2);
    $value = $mode === "resource" ? STDIN : [];
    return call_user_func("strlen", $value);
}

measure(defined("CALL_TEST_MODE") ? CALL_TEST_MODE : "empty");
