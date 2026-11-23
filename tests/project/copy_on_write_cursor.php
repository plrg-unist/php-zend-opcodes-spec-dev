<?php
$unshared = ["x" => 0, 8 => $argc];
unset($unshared["x"], $unshared[8]);
$unshared[0] = 8;
$unshared[] = 3;
echo (int) isset($unshared[1]), (int) isset($unshared[9]), ":";
$shared = ["x" => 0, 8 => $argc];
unset($shared["x"], $shared[8]);
$alias = $shared;
$shared[0] = 8;
$shared[] = 3;
echo (int) isset($shared[1]), (int) isset($shared[9]), count($alias), ":";
$released = ["x" => 0, 8 => $argc];
unset($released["x"], $released[8]);
$saved = $released;
unset($saved);
$released[0] = 8;
$released[] = 3;
echo (int) isset($released[1]), (int) isset($released[9]), "\n";
for ($release = 0; $release < 2; ++$release) {
    $inner = ["x" => 0, 8 => $argc];
    unset($inner["x"], $inner[8]);
    $outer = [$inner];
    unset($inner);
    $other = $outer;
    $outer["x"] = 1;
    if ($release) {
        unset($other);
    }
    $outer[0][0] = 8;
    $outer[0][] = 3;
    echo (int) isset($outer[0][1]), (int) isset($outer[0][9]), "\n";
}
