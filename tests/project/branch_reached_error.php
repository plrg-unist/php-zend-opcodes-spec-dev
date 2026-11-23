<?php
$zero = $argc - 1;
$null = $zero ? $argc : null;
$skipped = $argc ?: $argc / $zero;
echo $skipped, "\n";
echo $null ?? $argc / $zero;
echo "unreachable";
