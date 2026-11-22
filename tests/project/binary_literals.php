<?php
$values = [
    -2 => [0 => "quote\"\\\0\xff", "01" => true, 1 => false],
    "key\0" => [-0.0, 1.0000000000000002, null, -17],
];
echo serialize($values), "\n";
