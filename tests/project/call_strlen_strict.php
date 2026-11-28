<?php
declare(strict_types=1);
function lengthOf($value) { return call_user_func('strlen', $value); }
echo "binary=", lengthOf("A\0\xff"), "\n";
echo lengthOf(true);
