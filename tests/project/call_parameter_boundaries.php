<?php
function callTarget($mode) {
    echo "before|";
    $value = 7;
    $list = [7];
    switch ($mode) {
    case "named": receiveNamed($value); break;
    case "union": receiveUnion($value); break;
    case "intersection": receiveIntersection($value); break;
    case "ref-dimension": receiveReference($list[0]); break;
    case "ref-literal": receiveReference(7); break;
    case "builtin": call_user_func("bin2hex", $value); break;
    case "variadic": receiveVariadic($value, $value); break;
    default: receiveNone($value, $value);
    }
}
function receiveNamed(FirstClass $value) { echo "named"; }
function receiveUnion(FirstClass|SecondClass $value) { echo "union"; }
function receiveIntersection(FirstClass&SecondClass $value) { echo "intersection"; }
function receiveReference(&$value) { echo "reference"; }
function receiveVariadic($first, ...$rest) { echo "variadic"; }
function receiveNone() { echo "no parameters"; }

callTarget(defined("CALL_TEST_MODE") ? CALL_TEST_MODE : "surplus");
