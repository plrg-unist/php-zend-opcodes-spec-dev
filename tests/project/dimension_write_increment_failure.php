<?php
$a = [[$argc]];
echo $a[0][0], ":";
$previous = $a[0]++;
echo $previous;
