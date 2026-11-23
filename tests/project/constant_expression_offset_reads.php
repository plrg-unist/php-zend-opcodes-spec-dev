<?php
const RECORD = [1 => "one", "01" => "text", "nil" => null];
const BINARY_TEXT = "a\0\xff";
const EMPTY_ARRAY = [];
const FLAG = false;

const OBSERVED = [
    RECORD["1"], RECORD["01"], RECORD["nil"] ?? "fallback", RECORD["missing"] ?? "quiet",
    RECORD["raw\0\xff"], BINARY_TEXT[-1], BINARY_TEXT[true], BINARY_TEXT["1suffix"] ?? "unused",
    BINARY_TEXT[-4], BINARY_TEXT[8] ?? "outside", BINARY_TEXT["invalid"] ?? "invalid",
    FLAG["field"], FLAG["field"] ?? "scalar", EMPTY_ARRAY->{EMPTY_ARRAY}, FLAG?->{"field\0x"},
];
foreach (OBSERVED as $value) {
    echo $value === null ? "<null>" : $value, "|";
}
echo "\n";
