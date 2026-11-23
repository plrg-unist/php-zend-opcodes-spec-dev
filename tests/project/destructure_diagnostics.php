<?php
$source = [1 => "one", "" => "empty"];
["missing\0\xff" => [$missing], $missingKey => $empty, $argc + 0.5 => $float] = $source;
echo (int) ($missing === null), "|", $empty, "|", $float, "\n";
[$missingKey => $fromMissing] = $missingSource;
echo (int) ($fromMissing === null), "\n";

[[] => $string] = "\xff\0";
[$missingKey => $integer] = $argc;
[$boolean] = false;
[$otherBoolean] = true;
[$double] = 1.5;
[[] => $nothing] = null;
echo (int) ($string === null), (int) ($integer === null), (int) ($boolean === null),
    (int) ($otherBoolean === null), (int) ($double === null), (int) ($nothing === null), "\n";
