<?php
$arr = ["old" => &$missing, "alias" => &$missing, "same" => &$missing, "same" => $argc];
$arr["old"] = 4;
$snapshot = $arr;
$snapshot["alias"] += 3;
$snapshot["same"] += 2;
echo $arr["old"], $arr["alias"], $arr["same"], ":", $snapshot["alias"], $snapshot["same"], $missing, "\n";
$keep = $missing;
unset($arr["old"], $snapshot["alias"], $missing);
$arr["alias"] = 2;
echo $snapshot["old"], $keep, ":", (int)isset($missing), "\n";
