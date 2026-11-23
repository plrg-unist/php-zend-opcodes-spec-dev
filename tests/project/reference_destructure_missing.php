<?php
[&$first, &$last] = $new;
$first = 7;
$last = 8;
echo $new[0], $new[1], ":";
$new[] = 9;
[$key => &$missing] = $new;
$missing = "x";
echo $new[2], $new[""], "\n";
$previous = false;
[&$a] = $previous;
$a = 3;
echo $previous[0], "\n";
$nested = [];
[[&$value]] = $nested;
$value = 5;
echo $nested[0][0], "\n";
