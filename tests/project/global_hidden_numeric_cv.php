<?php
if (false) {
    echo ${'42'}, $a, $b, $c, $d, $e;
}
unset($GLOBALS['argv'], $GLOBALS['argc'], $GLOBALS['_GET'], $GLOBALS['_POST'],
    $GLOBALS['_COOKIE'], $GLOBALS['_FILES'], $GLOBALS['_SERVER']);
$snapshot = $GLOBALS;
$snapshot[9] = 'removed';
unset($snapshot[9]);
$snapshot[0] = 'first';
$snapshot[] = 'second';
foreach ($snapshot as $key => $value) {
    echo $key, ':', $value, "\n";
}
