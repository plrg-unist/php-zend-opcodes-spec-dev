<?php
$key = [$argc];
echo "before\n";
[0 => $first, $key => $unreached] = [11, 22];
echo $first, "|", $unreached, "\n";
