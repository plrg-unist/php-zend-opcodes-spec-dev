<?php
const EMPTY_RECORD = [];
echo "before\n";
const BUILT = [
    EMPTY_RECORD["key first"] => EMPTY_RECORD["value second"],
    EMPTY_RECORD => EMPTY_RECORD["value before key conversion"],
    NEVER_REACHED,
];
