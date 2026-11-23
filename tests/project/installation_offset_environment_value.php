<?php
namespace Offset;
$path = PHP_BINARY;
$environment = $_ENV;
$value =& $environment["ZEND_WRITE_BYTE"];
echo "value|";
$path[0] = $value;
