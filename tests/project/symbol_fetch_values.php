<?php
$name = 'item';
$item = [1, 2];
$copy = $$name;
$$name[0] = 3;
unset($$name[1]);
echo $copy[0], $copy[1], ':', $item[0], (int)isset($item[1]), "\n";
$ref =& $$name;
$$name = 4;
echo $ref, ':', $$name, ':';
++$$name;
echo $ref, ':', $$name, "\n";
unset($$name);
echo $ref, ':', (int)isset($item), ':', (int)empty($$name), "\n";
$$name = null;
echo (int)isset($$name), ':', (int)empty($$name), ':', $$name ?? 'fallback', "\n";
$$name = 7;
if (isset($$name)) {
    echo 'set';
}
if (!empty($$name)) {
    echo ' nonempty';
}
echo ':', $item, "\n";
$name = 'dynamic';
$other =& $$name;
$$name = 8;
echo $other, ':', $GLOBALS[$name], "\n";
unset($GLOBALS[$name]);
echo $other, ':', (int)isset($$name), "\n";
