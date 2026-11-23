<?php
$values = [3];
$key = [$argc];
echo $values[0], ":";
[$key => &$ref] = $values;
