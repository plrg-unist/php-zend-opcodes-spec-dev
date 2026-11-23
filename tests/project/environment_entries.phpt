--TEST--
The CLI environment and server keep separate writable aliases to an inherited key and a clock
--FILE--
<?php
$env = $_ENV;
echo (int) is_array($env), (int) isset($env['']), (int) isset($env['a.b']), (int) isset($env["a\0b"]), "|";
$env['a.b'] = 0;
echo (int) (bool) $env, (int) empty($env), $env['a.b'], (int) isset($_ENV['a.b']), "|";
$copy = $env;
$copy['a.b'] = 5;
unset($copy['any']);
echo $copy['a.b'], $env['a.b'], (int) isset($copy['any']), "|";
$env['argv'] = ['local'];
$env['REQUEST_TIME'] = 'known';
echo $env['argv'][0], ':', $env['REQUEST_TIME'], ':', (int) is_array($_SERVER['argv']), '|';
$clock =& $_SERVER['REQUEST_TIME'];
$value =& $_ENV['REQUEST_TIME'];
echo (int) isset($clock), (int) is_int($clock), (int) is_int($value), (int) is_float($value), '|';
$left = $_SERVER;
$right = $_SERVER;
$left['a.b'] =& $value;
$right['a.b'] =& $clock;
echo (int) ($left === $right);
$right['a.b'] =& $value;
echo (int) ($left === $right), '|';
$value = 4;
echo $value, (int) isset($_ENV['REQUEST_TIME']), "\n";
--EXPECT--
1000|1000|500|local:known:1|1100|01|41
