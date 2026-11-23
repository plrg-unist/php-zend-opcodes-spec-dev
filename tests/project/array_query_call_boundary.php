<?php
$values = [$argc, "two"];
echo count($values), ":";
echo (int) in_array($argc, $values, true);
