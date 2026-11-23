<?php
$names = ['', false, null, true, 1, '1', '01', -4, 2.5, INF, ['a'], 'Array', 'before', "before\0after", 'GLOBALS'];
$i = 0;
foreach ($names as $name) {
    $$name = ++$i;
}
foreach ($names as $name) {
    echo $$name, ':', (int)isset($$name), ' ';
}
echo "\n";
$original = 'from_reference';
$name =& $original;
$$name = 'yes';
echo $from_reference, ':', $$name, "\n";
$name = NAN;
$$name = 'nan';
echo $$name, "\n";
unset($$name);
echo (int)empty($$name), "\n";
$name = STDOUT;
$$name = 'stream name';
$spelling = 'Resource id #2';
echo $$spelling, ':', $$name, "\n";
$name = 'not_a_number';
$not_a_number = NAN;
$alias =& $not_a_number;
echo (int)isset($$name), ':', (int)empty($$name), "\n";
