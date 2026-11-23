<?php
$int = $argc;
$bool = $argc === 1;
$float = $argc + 0.25;
echo gettype($unknown[$key]), "\n";
echo $unknown[$key] ?? "fallback", "|", (int) isset($unknown[$key]), (int) empty($unknown[$key]), "\n";
echo gettype($int[[]]), "|", gettype($bool[1]), "|", gettype($float[-1]), "\n";
echo $int[$key] ?? "unread", "|", (int) isset($int[[]]), (int) empty($float[[]]), "\n";
