<?php
$time = &$_SERVER['REQUEST_TIME'];
echo "before\n";
echo match ($time) {
    1 => "integer",
    "candidate" => "string",
    default => "default",
};
