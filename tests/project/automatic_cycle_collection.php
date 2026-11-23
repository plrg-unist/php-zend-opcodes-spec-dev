<?php
$value = 1;
$original = [&$value];
$cycle = [];
$cycle[0] = &$cycle;
$cycle[1] = &$value;
unset($cycle, $value);

// Keep the old arrays reachable as the chain passes Zend's 10,000-candidate threshold.
$chain = [];
for ($i = 0; $i < 1280; ++$i) {
    $previous = $chain;
    $chain = [&$previous];
    unset($previous);
    $chain = [$chain];
    $chain = [$chain];
    $chain = [$chain];
    $chain = [$chain];
    $chain = [$chain];
    $chain = [$chain];
    $chain = [$chain];
}
unset($chain);

$copy = $original;
$copy[0] = 7;
echo $original[0], $copy[0];
