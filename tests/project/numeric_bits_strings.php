<?php
$long = $argc ? "\x00\xffA\x80" : "unreached";
$short = "\xff\x0f";
$empty = "";
$one = "\x80";
echo $long & $short, ":", $short & $long, ":", $long | $short, ":", $short | $long, "\n";
echo $long ^ $short, ":", $short ^ $long, ":", ~$long, ":", ~$one, ":", ~$empty, "\n";
echo $empty & $long, ":", $long | $empty, ":", $empty | $long, ":", $long ^ $empty, "\n";
echo $one & "\x0f", ":", $one | "\x0f", ":", $one ^ "\x0f", ":", "12" & "03", ":", "2" ^ "1", "\n";
$a = $long;
echo gettype($a &= $short), ":", $a, ":", ($a |= $long), ":", $a, ":", ($a ^= $a), ":", $a, "\n";
echo $long, ":", $short, "\n";
