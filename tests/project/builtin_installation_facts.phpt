--TEST--
The existence, type, and identity of predefined installation strings survive copies
--FILE--
<?php
namespace Install;
const COPY = PHP_BINARY;
echo (int) \defined("PHP_BINARY"), (int) \defined("PEAR_EXTENSION_DIR"), "\n";
$path = COPY;
echo \gettype($path), "/", (int) \is_string($path), "/", (int) isset($path), "/", (int) ($path === PHP_BINARY), "\n";
$first = [PHP_EXTENSION_DIR];
$second = [PEAR_EXTENSION_DIR];
echo (int) ($first === $second), (int) ($first[0] == $second[0]), (int) isset($first[0]), "\n";
echo (int) ($path !== 1), (int) (false !== $path), (int) ([] > $path), "\n";
$_SERVER['PHP_SELF'] = PHP_EXTENSION_DIR;
$server = $_SERVER;
$server['PHP_SELF'] = PEAR_EXTENSION_DIR;
echo (int) ($server === $_SERVER), "\n";
$ref = &$path;
$coalesced = $ref ?? null;
$offset = $second[0] ?? false;
$cast = (string) $path;
$wrapped = (array) $path;
echo (int) ($coalesced === PHP_BINARY), (int) ($offset === PHP_EXTENSION_DIR), (int) ($cast === $path),
     "/", \count($wrapped), "/", (int) ($wrapped[0] === $path), "\n";
$clock = &$_SERVER['REQUEST_TIME'];
echo (int) ([1] !== [$path]), (int) ($clock !== $path), "\n";
$nan = NAN;
echo $path <=> $nan, "/", $nan <=> $path, "/", (int) ([$path] != [$nan]), (int) ($nan < $path), "\n";
$ignored = PHP_BUILD_DATE;
$ignored = 5;
echo $ignored, "\n";
--EXPECT--
11
string/1/1/1
111
111
1
111/1/1
11
1/1/10
5
