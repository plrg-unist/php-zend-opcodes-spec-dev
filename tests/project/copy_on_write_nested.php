<?php
$leaf = [NAN * $argc, "v" => 1];
$source = ["left" => $leaf, "right" => $leaf];
$copy = $source;
echo (int) ($source === $copy), (int) ($source["left"] === $source["right"]), "\n";
$copy["left"]["v"] = $argc + 1;
echo $source["left"]["v"], $copy["left"]["v"], $copy["right"]["v"], ":";
echo (int) ($source["left"] === $copy["left"]), (int) ($source["right"] === $copy["right"]), "\n";
$source["right"][] = "new";
echo count($source["right"]), count($copy["right"]), count($leaf), ":";
echo (int) ($source["left"] === $leaf), (int) ($copy["right"] === $leaf), "\n";
unset($copy["left"]["v"]);
echo (int) isset($copy["left"]["v"]), $source["left"]["v"], $leaf["v"], "\n";
