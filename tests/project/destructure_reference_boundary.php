<?php
$data = [[$argc]];
[$prefix] = $data[0];
echo $prefix, "|";
[[&$value]] = $data;
$value = 12;
echo $data[0][0], "\n";
