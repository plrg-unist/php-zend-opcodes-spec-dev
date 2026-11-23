<?php
namespace Offset;
$path = PHP_BINARY;
foreach (["bad", "7tail", [], \STDIN] as $key) {
    echo (int) isset($path[$key]), (int) empty($path[$key]), "|";
}
echo $path["bad"] ?? "default", "\n";
echo $path[\STDIN] ?? "unreachable";
