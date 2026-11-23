<?php
$box = ["pair" => [$argc, 2], "spare" => null];
$copy = $box;
$key = "pair";
[$key => [0 => &$a, 1 => $b], "spare" => &$slot, $key => [0 => &$again]] = $box;
$a = 4;
$again += 2;
$b = 8;
$slot = [$again];
echo $a, $again, $b, ":", $box["pair"][0], $box["pair"][1], $box["spare"][0],
    ":", $copy["pair"][0], $copy["pair"][1], (int) isset($copy["spare"]), "\n";
[$a, &$b] = $box["pair"];
$b++;
echo $again, $b, $box["pair"][1], "\n";
