<?php
$data = [$argc => null];
echo "isset\n";
echo (int) isset($data[[$argc]]);
echo "unreachable";
