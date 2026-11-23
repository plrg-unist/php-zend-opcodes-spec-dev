<?php
$none = $argc === 1 ? null : $argc;
$empty = (array) $none;
$binary = "a\0" . $argc;
$values = [(array) $argc, (array) ($argc + 0.25), (array) ($argc > 0),
           (array) ($argc < 0), (array) $binary];
echo (int) ($empty === []), (int) ($values === [[1], [1.25], [true], [false], ["a\x001"]]), "\n";
echo (int) $empty, (int) ((float) $empty === 0.0), (int) (bool) $empty, "\n";
echo (int) $values, (int) ((float) $values === 1.0), (int) (bool) $values, "\n";
$nan = [NAN * $argc];
$copy = (array) $nan;
echo (int) ($copy === $nan), (int) (((array) $nan)[0] === $nan[0]), "\n";
$boxed = (array) $nan[0];
echo count($boxed), ":", (int) ($boxed === $nan), ":", gettype($boxed[0]), "\n";
$absent = (array) $missing;
echo (int) ($absent === []), ":", (string) $empty, ":", (string) $values, "\n";
