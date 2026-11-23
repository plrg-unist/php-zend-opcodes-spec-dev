<?php
$array = [$argc];
[&$ref] = $array;
$ref++;
echo $array[0], ":";
$name = $argc ? "ref" : "other";
echo $$name;
