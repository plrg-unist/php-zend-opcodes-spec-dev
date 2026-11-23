<?php
$data = [$argc => null, "01" => false, "" => true, "b\0" => [], -4 => "end"];
echo count($data), "|", sizeof($data), "|", gettype($data), "|", (int) is_array($data), (int) is_array($argc), "\n";
echo (int) array_key_exists((string) $argc, $data), (int) isset($data[$argc]),
     (int) array_key_exists("01", $data), (int) array_key_exists("b\0", $data),
     (int) array_key_exists("absent", $data), "\n";
echo (int) array_key_exists(true, $data), (int) array_key_exists(false, $data),
     (int) array_key_exists($argc + 0.5, $data), "\n";
$key = $argc ? null : false;
echo (int) array_key_exists($key, $data), (int) array_key_exists($undefined, $data), "\n";
if (array_key_exists("b\0", $data) && !array_key_exists("b", $data)) {
    echo "binary\n";
}
