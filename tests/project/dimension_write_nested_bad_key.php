<?php
$a = [$argc];
echo $a[0], ":";
$a[[$argc]][] = "new";
echo "reached";
