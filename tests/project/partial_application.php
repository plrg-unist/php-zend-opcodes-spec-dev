<?php
function triple($first, $middle, $last) {
    return "$first/$middle/$last";
}

function showPartial($partial) {
    $captured = (new ReflectionFunction($partial))->getClosureUsedVariables();
    echo $partial("x", "y"), " captures=", count($captured), "\n";
}

showPartial(triple(?, "middle", ?));
showPartial(triple("first", last: ?, middle: ?));
showPartial(triple("first", ...));

function appendBound($value, $element) {
    $value["items"][] = $element;
    return implode(",", $value["items"]);
}

$partial = appendBound(["items" => ["seed", "two"]], ?);
$items = ["items" => ["seed", "two"]];
$items["items"][] = "outside";
echo $partial("x"), " ", $partial("y"), " ", implode(",", $items["items"]),
    " captures=", count((new ReflectionFunction($partial))->getClosureUsedVariables()), "\n";
