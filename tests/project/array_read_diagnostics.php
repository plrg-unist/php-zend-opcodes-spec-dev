<?php
$data = [0 => "zero", $argc => "one", "" => "empty-key"];
echo $data[$argc + 8] === null ? "N\n" : "wrong\n";
echo $data["b\"\0\xff"] === null ? "B\n" : "wrong\n";
echo $data[$argc + 0.5], "|", $data[$argc * 1.0], "|", $data[true], "\n";
$nan = $argc ? NAN : 1;
echo $data[$nan], "|", (int) array_key_exists($nan, $data), "\n";
echo $data[$unsetKey], "|", $data[$unsetKey] ?? "wrong", "\n";
echo (int) isset($data[$unsetKey]), (int) empty($data[$unsetKey]), "\n";
$null = $argc ? null : 0;
echo $data[$null], "|", $data[$null] ?? "wrong", "|", (int) isset($data[$null]), "\n";
echo gettype($data["absent"][0]), "\n";
