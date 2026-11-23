--TEST--
Removing a dynamic symbol releases its reference while the last array owner survives
--FILE--
<?php
$name = "dynamic";
$value = [$argc];
$$name =& $value;
$bucket = [&$value];
unset($value);

$copy = $bucket;
$copy[0][] = 2;
echo count($bucket[0]), ":", $$name[1], "\n";
unset($copy, $$name);
$copy = $bucket;
$copy[0][] = 3;
echo count($bucket[0]), ":", count($copy[0]), ":", $copy[0][2], "\n";

$$name =& $copy[0];
$last = [&$$name];
unset($$name, $copy);
$$name = [4];
$other = $last;
$other[0][0] = 5;
echo $last[0][0], ":", $other[0][0], ":", $$name[0], "\n";
--EXPECT--
2:2
2:3:3
1:5:4
