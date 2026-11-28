<?php
function lengthOf($value) { return call_user_func('strlen', $value); }
foreach (["a\0\xff", 12, -1.25, false, true, null, NAN] as $value) {
    echo lengthOf($value), "\n";
}
lengthOf(null);
echo "discarded\n";
