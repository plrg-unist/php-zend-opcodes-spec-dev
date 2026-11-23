<?php
$key = $argc;
$a = [$key => ["number" => 5, "text" => "p"]];
$copy = $a;
$one = ($a[$key]["number"] += $key);
$two = ($a[$key]["number"] *= 0.5);
$three = ($a[$key]["text"] .= 12);
$four = ($a[$key]["number"] **= 2);
echo (int) ($one === 6), (int) ($two === 3.0), (int) ($three === "p12"), (int) ($four === 9.0), ":";
echo (int) ($a[$key] === ["number" => 9.0, "text" => "p12"]), (int) ($copy[$key] === ["number" => 5, "text" => "p"]), "\n";
$old = $a[$key]["number"]++;
$now = --$a[$key]["number"];
echo $old, ":", $now, ":", $a[$key]["number"], "\n";
