--TEST--
Global snapshots preserve shared references and separate references with a single owner
--FILE--
<?php
$x = 4;
$alias = &$x;
$sole = 7;
$holder = &$sole;
unset($holder);
$name = 'via';
$$name = &$x;

$snapshot = $GLOBALS;
$snapshot['sole'] = 71;
$snapshot['via'] = 42;
echo $sole, ':', $snapshot['sole'], ':', $x, ':', $alias, ':', $$name, "\n";
unset($snapshot, $alias, $$name);
$next = $GLOBALS;
$next['x'] = 43;
echo $x, ':', $next['x'], "\n";

$plain = 1;
$shared = 2;
$held = &$shared;
foreach ($GLOBALS as $key => &$value) {
    if ($key === 'plain' || $key === 'shared') {
        $value += 2;
    }
}
unset($value);
echo $plain, ':', $shared, ':', $held, "\n";

$cycle = [];
$cycle[0] = &$cycle;
$name = 'root';
$$name = &$cycle[0];
$snapshot = $GLOBALS;
$snapshot['root'][1] = 21;
unset($cycle, $$name);
echo $snapshot['cycle'][0][1], ':', $snapshot['root'][0][1], "\n";
--EXPECT--
7:71:42:42:42
42:43
1:4:4
21:21
