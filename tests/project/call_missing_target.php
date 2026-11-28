<?php
function tap() { echo "tap"; return "argument"; }
echo "start|";
unregisteredTarget(tap());
