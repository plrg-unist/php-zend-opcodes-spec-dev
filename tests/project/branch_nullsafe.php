<?php
declare(strict_types=1);
$steps = 0;
$receiver = $argc === 1 ? null : $argc;
$value = $receiver?->missing(++$steps)?->{strlen($argc)};
$isset = isset($receiver?->{++$steps}->next);
$empty = empty($receiver?->{++$steps}->next);
$fallback = $missing?->next ?? ++$steps;
echo gettype($value), ":", gettype($isset), (int) $isset, ":", gettype($empty), (int) $empty, "\n";
echo $fallback, ":", $steps, "\n";
$read = $missing?->next;
echo gettype($read), "\n";
