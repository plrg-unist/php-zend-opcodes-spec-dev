<?php
const LAST = PHP_INT_MAX;
echo "unpack\n";
const TOO_MANY = [LAST => 1, ...["s" => 2, 40 => 3, 41 => 4], NEVER_REACHED];
