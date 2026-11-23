<?php
$first = [];
$first[0] = &$first;
$second = [];
$second[0] = &$second;
$env = $_ENV;
$env["X"] = $first;
$left = $_ENV;
$right = $left;
$left["inner.key"] = ["X" => $second];
$right["inner.key"] = $env;
$left["last.key"] = 1;
$right["last.key"] = 2;
echo "nested|", (int) ($left == $right);
