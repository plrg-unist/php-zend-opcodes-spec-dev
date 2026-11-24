<?php
namespace Install;

const OPTIONS = ["key" => "found", "" => "empty"];
const PICK = OPTIONS[PHP_BINARY] ?? "missing";
const BYTE = PHP_BINARY[1.5];
const PROPERTY = PHP_BINARY->{PHP_EXTENSION_DIR};
echo PICK, "|", BYTE, "|", (int) (PROPERTY === null), "\n";

$path = PHP_BINARY;
$copy = $path;
$ref = &$path;
echo ($ref[0] = "Q"), "|", $path, "|", $copy, "|", PHP_BINARY, "\n";
