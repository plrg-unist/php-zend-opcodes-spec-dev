<?php
namespace Offset;
$path = PHP_BINARY;
$environment = $_ENV;
$key =& $environment["ZEND_STRING_OFFSET"];
echo "offset|";
echo $path[$key];
