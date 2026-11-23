<?php
$first = [];
$first[0] =& $first;
$second = [];
$second[0] =& $second;
$env = $_ENV;
$env['X'] = $first;
$plain = ['never.key' => 1, 'X' => $second];
echo 'unordered|', (int) ($env == $plain);
