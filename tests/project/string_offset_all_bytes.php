<?php
$low = "\x00\x01\x02\x03\x04\x05\x06\x07\x08\x09\x0a\x0b\x0c\x0d\x0e\x0f";
$high = "\x00\x10\x20\x30\x40\x50\x60\x70\x80\x90\xa0\xb0\xc0\xd0\xe0\xf0";
$s = "";
for ($i = 0; $i < 256; $i++) {
    $byte = $low[$i & 15] | $high[$i >> 4];
    echo ($s[$i] = $byte);
}
echo $s;
for ($i = 1; $i <= 256; $i++) {
    echo $s[-$i];
    if (empty($s[-$i])) {
        echo "E";
    }
}
