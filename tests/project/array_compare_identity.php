<?php
$nan = NAN * $argc;
for ($i = 0; $i < 2; ++$i) {
    $candidate = [$nan];
    if ($i === 0) {
        $saved = $candidate;
    }
}
$copy = $saved;
echo (int) ($saved === $copy), (int) ($saved == $copy), ":", $saved <=> $copy, "\n";
echo (int) ($saved !== $candidate), (int) ($saved != $candidate), ":", $saved <=> $candidate, "\n";
echo (int) ($saved <= $candidate), (int) ($candidate <= $saved), "\n";

$outer = [$saved, $argc];
$shared = [$copy, $argc];
$fresh = [$candidate, $argc];
echo (int) ($outer === $shared), (int) ($outer == $shared), ":", $outer <=> $shared, "\n";
echo (int) ($outer === $fresh), (int) ($outer == $fresh), ":", $outer <=> $fresh, "\n";
echo (int) ($outer[0] === $copy), (int) ($outer[0] === $candidate), "\n";
$selected = $saved ?: $candidate;
$unpacked = [...[$selected ?? $candidate]];
echo (int) ($unpacked[0] === $saved), (int) ($unpacked[0] === $candidate), "\n";
