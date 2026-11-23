--TEST--
Matching CLI environment and server overwrites preserve insertion order
--FILE--
<?php
$left = $_ENV;
$right = $_SERVER;
$keys = ["PHP_SELF", "SCRIPT_NAME", "SCRIPT_FILENAME", "PATH_TRANSLATED", "DOCUMENT_ROOT",
          "REQUEST_TIME_FLOAT", "REQUEST_TIME", "argv", "argc"];
foreach ($keys as $key) {
    $left[$key] = $right[$key] = 0;
}
echo (int) ($left === $right), (int) ($left == $right), $left <=> $right, "|";
unset($right['PHP_SELF'], $left['PHP_SELF']);
$right['PHP_SELF'] = $left['PHP_SELF'] = null;
echo (int) ($left === $right), (int) ($left == $right), $left <=> $right, "\n";
--EXPECT--
110|110
