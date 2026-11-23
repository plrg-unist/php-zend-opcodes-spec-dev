--TEST--
A by-value foreach reads a cyclic element before the referenced array separates
--FILE--
<?php
$a = [];
$a["self"] =& $a;
$a["n"] = 1;
$copy = $a;
foreach ($copy as $key => $value) {
    if ($key === "self") {
        $a["later"] = 2;
        echo count($value), count($copy[$key]), count($copy), ":";
    } else {
        echo $key, $value;
    }
}
echo "\n";
--EXPECT--
232:n1
