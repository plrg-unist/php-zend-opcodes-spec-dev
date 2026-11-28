<?php
function updateLocals($name, $globalName) {
    $local = 5;
    $$name = 7;
    global $total;
    $total += $local;
    $snapshot = $GLOBALS;
    $GLOBALS[$globalName] += 3;
    unset($GLOBALS['gone']);
    echo $$name, ':', $snapshot['local'], ':', isset($snapshot['globalName']), ':', $total,
        ':', isset($argc), ':', $_SERVER['argc'], "\n";
    unset($$name);
    echo isset($$name), ':', $GLOBALS['local'], ':', isset($GLOBALS['gone']), "\n";
}

$local = 100;
$total = 12;
$x = 3;
$alias = &$x;
$gone = 4;
updateLocals('local', 'x');
echo $local, ':', $total, ':', $x, ':', $alias, ':', isset($gone), "\n";
