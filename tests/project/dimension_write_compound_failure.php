<?php
$a = [$argc => 3];
echo $a[$argc], ":";
$a[$argc] /= $argc - 1;
echo "reached";
