<?php
$text = 'abc';
$key =& $_SERVER['REQUEST_TIME'];
unset($text[$key]);
echo 'after';
