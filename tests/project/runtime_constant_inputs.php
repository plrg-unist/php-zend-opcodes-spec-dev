<?php
namespace RuntimeInputs;

echo \GLOB_MARK, "/", GLOB_MARK, "\n";
echo \defined("STDOUT") ? \gettype(STDOUT) : "absent", "\n";
if (\defined("INPUT_BYTES")) {
    $value = \RuNtImEiNpUtS\LongNameExceedingTheLengthOfAnyReferenceCatalogConstant;
    echo \gettype($value), "/", $value, "/", (int) (INPUT_BYTES === "\x00\xff"), (int) (INPUT_BYTES === INPUT_BYTES_ALIAS),
         "/", (int) (INPUT_NAN !== INPUT_NAN), "/", \gettype(INPUT_STREAM), "/", (int) INPUT_STREAM, "\n";
}
if (\defined("INPUT=FLAG")) {
    echo "equals/", (int) \defined("INPUT\0FLAG"), "\n";
}
