<?php
function tap($value) { echo "tap"; return $value; }
echo "start|";
bin2hex(tap('binary'));
echo "|after";
