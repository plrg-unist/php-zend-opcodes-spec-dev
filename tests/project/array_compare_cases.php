<?php
$n = $argc;
switch ([$n]) {
    case [$n + 1]: echo "wrong"; break;
    case [(string) $n]: echo "loose"; break;
    case $unread: echo "wrong"; break;
}
echo ":", match ([$n]) {
    [(string) $n] => "wrong",
    [$n] => "strict",
    default => "wrong"
}, "\n";
