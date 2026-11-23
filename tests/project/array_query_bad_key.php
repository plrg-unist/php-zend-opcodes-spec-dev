<?php
$data = [$argc];
echo "exists\n";
echo (int) array_key_exists([$argc], $data);
echo "unreachable";
