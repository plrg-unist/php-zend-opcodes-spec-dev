<?php
$server = $_SERVER;
$plain = ["argv" => null];
for ($i = 0; $i < 8; ++$i) {
    $plain["absent." . $i] = 1;
}
$plain["argv"] = &$plain;
echo "argv|", (int) ($plain === $server);
