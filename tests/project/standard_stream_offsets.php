<?php
$r = STDOUT;
$map = [$r => 'a'];
$map[$r] .= 'b';
echo $map[2], '/', $map[$r], '/', (int) isset($map[$r]), '/', (int) array_key_exists($r, $map), "\n";
unset($map[$r]);
echo count($map), '/', $r[0] ?? 'missing', "\n";
foreach ($r as $value) {
    echo 'unexpected';
}
