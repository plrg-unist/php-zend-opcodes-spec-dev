<?php
$request = is_array($_REQUEST);
if (false) {
    echo $_ENV['unused'];
}

var_dump($request, is_array($_SERVER));
foreach (array_keys($GLOBALS) as $name) {
    if ($name === '_ENV' || $name === '_REQUEST') {
        echo $name, "\n";
    }
}
