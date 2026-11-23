<?php
$env = $_ENV;
$env['set.key'] = 1;
echo 'numeric|';
echo (int) ($env === ['7' => 'value', 'set.key' => 1]);
