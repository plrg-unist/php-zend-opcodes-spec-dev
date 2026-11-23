<?php
$name = 'this';
echo 'local=', $$name, ';global=', $GLOBALS[$name], "\n";
echo $$name ?? 'a', ':', $GLOBALS[$name] ?? 'b', "\n";
unset($$name, $GLOBALS[$name]);
echo (int)isset($$name), ':', (int)empty($$name), "\n";
