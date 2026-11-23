<?php
$data = [$argc => 12];
echo "read\n";
echo $data[[$argc]];
echo "unreachable";
