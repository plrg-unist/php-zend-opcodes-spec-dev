<?php
$inner = $_ENV;
$inner["item.key"] = [1];
$left = $_ENV;
$right = $left;
$left["first.key"] = ["item.key" => [1]];
$right["first.key"] = $inner;
$a = [];
$a[0] = &$a;
$b = [];
$b[0] = &$b;
$left["last.key"] = $a;
$right["last.key"] = $b;
echo "later|", (int) ($left === $right);
