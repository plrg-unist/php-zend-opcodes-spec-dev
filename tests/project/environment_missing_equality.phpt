--TEST--
Opaque array equality can stop at a key excluded by the CLI without comparing recursive values
--FILE--
<?php
$env = $_ENV;
foreach (['', 'a.b', 'a b', 'a[b', 'a=b', "a\0b"] as $key) {
    $plain = [$key => 1];
    echo (int) ($env !== $plain), (int) ($plain != $env);
}
echo '|';
$env['stored'] = 2;
unset($env['deleted']);
$plain = ['stored' => 2, 'deleted' => 'absent'];
echo (int) ($env === $plain), (int) ($plain == $env), '|';
$x = [];
$x[] =& $x;
$y = [];
$y[] =& $y;
$plain = ['ordinary' => $x, 'never.key' => 0];
echo (int) ($_ENV === $plain), (int) ($plain == $_ENV), '|';
$env['nested.key'] = $x;
$plain = ['never.key' => 0, 'nested.key' => $y];
echo (int) ($plain === $env), (int) ($env === $plain), (int) ($plain == $env), "\n";
--EXPECT--
111111111111|00|00|000
