<?php
function sample() { return [NAN]; }
function other_sample() { return [NAN]; }

$first = sample();
$same = sample();
$other = other_sample();
echo (int) ($first === $same), (int) ($first == $same), $first <=> $same, "\n";
echo (int) ($first === $other), (int) ($first == $other), $first <=> $other, "\n";

$nested = [sample(), sample(), other_sample()];
$i = $argc - 1;
echo (int) ($nested[$i] === $nested[$i + 1]), (int) ($nested[$i] === $nested[$i + 2]), "\n";
