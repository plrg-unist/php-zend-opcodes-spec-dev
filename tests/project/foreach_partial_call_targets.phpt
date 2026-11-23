--TEST--
Foreach before a dormant partial with returning and nonreturning call targets
--FILE--
<?php
function stopIteration(): never { exit; }
function unionResult(): Iterator|Countable { throw new Exception; }
function intersectionResult(): Iterator&Countable { throw new Exception; }
function namedResult(): Iterator { throw new Exception; }

$values = [1, 2, 3];
foreach ($values as &$value) { $value += 2; }
unset($value);
foreach ($values as $value) { echo $value; }
if ($argc === 42) {
    $callback = match ($argc) {
        1 => stopIteration(),
        2 => array_merge(['marker' => true], ?),
        3 => array_sum($values),
        default => exit(9),
    };
}
echo "\n";
--EXPECT--
345
