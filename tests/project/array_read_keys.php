<?php
$data = [0 => "zero", $argc => null, -1 => "negative", "01" => false,
         "b\0\xff" => "binary\0\xff", "nested" => [$argc, "0", []]];
$number = (string) -$argc;
echo $data[false], "|", $data[$number], "|", $data["b\0\xff"], "\n";
echo gettype($data["01"]), "|", gettype($data["1"]), "|", $data["nested"][0], "\n";
echo (int) isset($data[$argc]), (int) isset($data["absent"]), (int) isset($data["01"]), "\n";
echo (int) empty($data["01"]), (int) empty($data["nested"][1]), (int) empty($data["nested"][2]),
     (int) empty($data["nested"]), (int) empty($data["missing"]), "\n";
echo $data["absent"]["deep"] ?? "missing", "|", $data[$argc] ?? "null", "|",
     $data["nested"][1] ?? "wrong", "\n";
if (isset($data[$number]) && !empty($data[$number])) {
    echo "found\n";
}
$nan = [$argc => NAN];
echo (int) isset($nan[$argc]), (int) empty($nan[$argc]), "\n";
