<?php
$empty = $argc ? [] : [1];
$full = [$argc - 1];
echo (int) (bool) $empty, (int) (bool) $full, (int) !$empty, (int) !$full, "\n";
echo (int) ($empty xor $full), (int) ($full xor true), "\n";
$hits = 0;
$skip = $empty && ++$hits;
$take = $full && ++$hits;
$fallback = $empty ?: "fallback";
$kept = $full ?: ++$hits;
echo (int) $skip, (int) $take, ":", $hits, ":", $fallback, ":", $kept[0], "\n";
echo (int) ($empty == null), (int) ($full == true), (int) ($empty === false), "\n";
echo $empty <=> true, ":", null <=> $full, ":", NAN <=> $empty, ":", $full <=> "", "\n";
