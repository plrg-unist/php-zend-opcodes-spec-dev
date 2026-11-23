<?php
$first = [];
$first[0] = &$first;
$second = [];
$second[0] = &$second;
$env = $_ENV;
$env["X"] = $first;
$plain = ["X" => $second, "never.key" => 1];
echo "compare|", (int) ($plain === $env);
