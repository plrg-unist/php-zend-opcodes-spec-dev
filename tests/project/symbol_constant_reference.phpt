--TEST--
A namespace constant names a global reference shared with a snapshot
--FILE--
<?php
namespace {
    const SYMBOL = "global\0slot";
}
namespace Example {
    $name = SYMBOL;
    $state = [$argc];
    $$name =& $state;
    $snapshot = $GLOBALS;
    $snapshot[$name][] = \STDOUT;
    echo $state[0], ":", \gettype($GLOBALS[$name][1]), ":", (int)($$name[1] === \STDOUT), "\n";

    unset($GLOBALS[$name]);
    $state[0] = 8;
    echo $snapshot[$name][0], ":", (int)isset($$name), ":", (int)\defined("SYMBOL"), "\n";
}
--EXPECT--
1:resource:1
8:0:1
