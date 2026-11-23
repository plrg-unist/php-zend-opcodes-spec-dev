<?php
switch ($long) {
    case 0: echo "zero\n"; break;
    case 1: case 2: case 3: case 4: echo "wrong\n"; break;
    default: echo "wrong\n";
}
switch ($string) {
    case "x": echo "wrong\n"; break;
    case "": echo "empty\n"; break;
    default: echo "wrong\n";
}
echo match ($key) { 1, "1" => "wrong\n", default => "default\n" };
switch ($argc + 1) {
    case $loose: echo "wrong\n"; break;
    default: echo "loose\n";
}
echo match ((string) $argc) { $strict => "wrong\n", default => "strict\n" };
