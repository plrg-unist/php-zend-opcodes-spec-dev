--TEST--
Assigning an array to its own dimension through an alias creates a direct cycle
--FILE--
<?php
$x = ['n' => 1];
$alias = &$x;
$x['p'] = $alias;
echo $x['p']['p']['n'];
$x['n'] = 2;
echo $x['n'], $x['p']['n'], $x['p']['p']['n'], "\n";

$x = ['n' => 1];
$x[] = $alias;
echo $x[0][0]['n'];
$x['n'] = 2;
echo $x['n'], $x[0]['n'], $x[0][0]['n'], "\n";

$value = 1;
$copy = [&$value];
$x = [&$value];
$x['self'] = $alias;
unset($x, $alias, $value);
$other = $copy;
$other[0] = 7;
echo $copy[0], $other[0], "\n";
--EXPECT--
1211
1211
77
