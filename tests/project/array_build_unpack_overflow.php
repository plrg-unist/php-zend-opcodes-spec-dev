<?php
$max = 9223372036854775807;
$more = ["ok" => $argc, $argc];
echo "unpack\n";
$a = [$max => "occupied", ...$more];
echo count($a);
