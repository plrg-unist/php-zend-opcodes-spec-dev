<?php
$cyclic = [];
$cyclic[0] =& $cyclic;
$finite = [[]];
echo (int) ($finite === $cyclic), ":";
echo (int) ($cyclic === $finite);
