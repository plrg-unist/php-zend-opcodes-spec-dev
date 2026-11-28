<?php
function failWithHostValue($value) {
    echo "inside|";
    return 1 / 0;
}
function failWithPrivateValue(#[SensitiveParameter] $value) {
    echo "hidden|";
    return 1 / 0;
}
$value = &$_SERVER['ZEND_OPCODES_TEST_PROFILE'];
$mode = defined('CALL_TEST_MODE') ? CALL_TEST_MODE : "trace";
if ($mode === "builtin") {
    echo "builtin|";
    echo call_user_func('strlen', $value);
} elseif ($mode === "redact") {
    failWithPrivateValue($value);
} else {
    failWithHostValue($value);
}
