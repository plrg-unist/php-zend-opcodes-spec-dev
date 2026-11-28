<?php
function warningOnReturn() { return $missing; }
warningOnReturn();
echo "discarded\n";
echo warningOnReturn(), "used\n";
