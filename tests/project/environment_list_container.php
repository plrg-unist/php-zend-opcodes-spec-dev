<?php
$ambient =& $_ENV['ZEND_LIST_CONTAINER'];
echo 'container|';
['key' => &$value] = $ambient;
$value = 'x';
echo $ambient['key'], "\n";
