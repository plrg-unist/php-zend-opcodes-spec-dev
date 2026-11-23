<?php
$a = true;
echo "assign\n";
$a[$missingKey] = $missingRhs;
echo "reached";
