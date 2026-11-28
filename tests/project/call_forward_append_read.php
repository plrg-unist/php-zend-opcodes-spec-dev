<?php
function forwardRead() {
    echo "before|";
    if (defined('CALL_TEST_HOST')) $values = &$_SERVER['ZEND_OPCODES_TEST_PROFILE'];
    later($values[]);
}
function later($value) { echo $value; }
forwardRead();
