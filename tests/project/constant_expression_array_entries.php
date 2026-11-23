<?php
const START = "-2";
const SOURCE = [9 => "nine", "s" => "updated", "01" => "string", -7 => "negative"];
const EMPTY_ITEMS = [];
const MIXED_ITEMS = [
    START => "first", "s" => "original", ...SOURCE, "after", ...EMPTY_ITEMS,
];
foreach (MIXED_ITEMS as $key => $value) {
    echo $key, "=", $value, "|";
}
echo "\n";

const NIL = null;
const FRACTION = 1.5;
const CONVERTED_KEYS = [NIL => "empty", FRACTION => "int", STDOUT => "resource"];
echo CONVERTED_KEYS[""], "|", CONVERTED_KEYS[1], "|", CONVERTED_KEYS[2], "\n";

const INNER = [NAN];
const PARTS = [INNER, "same" => INNER];
const MERGED = [...PARTS, "other" => [NAN]];
$copy = MERGED;
$copy[0][0] = "changed";
echo (int) (MERGED[0] === MERGED["same"]), (int) (MERGED[0] === INNER),
     (int) (MERGED[0] === MERGED["other"]), "|", $copy[0][0], "|",
     (int) (MERGED[0][0] !== MERGED[0][0]), "\n";
