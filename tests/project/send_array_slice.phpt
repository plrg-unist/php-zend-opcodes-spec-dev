--TEST--
Call with array arguments sliced using a variable or computed length
--FILE--
<?php
function joinArgs(...$args) { return implode(',', $args); }

function sliced($values, $length) {
    echo call_user_func_array('joinArgs', array_slice($values, 1, $length)), "\n";
    echo call_user_func_array('joinArgs', array_slice($values, 0, $length + 1)), "\n";
}

sliced(['a', 'b', 'c', 'd'], 2);
--EXPECT--
b,c
a,b,c
