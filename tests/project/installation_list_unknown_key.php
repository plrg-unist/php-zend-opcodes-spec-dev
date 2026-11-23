<?php
namespace Iteration;
$path = PHP_BINARY;
$environment = $_ENV;
$key =& $environment["ZEND_LIST_OFFSET"];
echo "key|";
[$key => &$item] = $path;
