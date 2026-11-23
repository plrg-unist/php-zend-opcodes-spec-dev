<?php
function local_results($v) {
    $array = [0, $v];
    $first = $array["1"] = $v;
    $second = ($array[$v] += $v);
    echo $first, $second, $array[1], "\n";
}
$array = [0, $argc];
$first = $array["1"] = $argc;
$second = ($array[$argc] += $argc);
echo $first, $second, $array[1], "\n";
