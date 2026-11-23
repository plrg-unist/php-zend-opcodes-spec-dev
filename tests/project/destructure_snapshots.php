<?php
$source = [$argc + 1, $argc + 2];
$whole = ([$source, $tail] = $source);
echo $source, "|", $tail, "|", $whole[0], "|", $whole[1], "\n";

$entry = [NAN * $argc];
$pair = [$entry, $entry];
[$first, $second] = $pair;
echo (int) ($first === $entry), (int) ($second === $first), "\n";
$first[1] = 42;
echo $first[1], "|", (int) ($first === $second), (int) ($second === $entry),
    (int) ($pair[0] === $entry), (int) isset($pair[0][1]), "\n";

$nested = [[$argc, 2], [3, 4]];
[[$nested[1][1], $nested[1][0]], [$old0, $old1]] = $nested;
echo $nested[1][0], "|", $nested[1][1], "|", $old0, "|", $old1, "\n";
