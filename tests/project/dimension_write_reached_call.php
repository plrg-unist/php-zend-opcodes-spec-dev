<?php
function supply($v) { echo "call"; return $v + 8; }
$a = [$argc => 4];
echo $a[$argc] ??= supply($argc), ":";
echo $a["missing"] ??= supply($argc);
