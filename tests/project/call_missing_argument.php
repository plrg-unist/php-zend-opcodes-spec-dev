<?php
function needsTwo($a, $b) { echo "body"; return $a + $b; }
function outerMissing($a) { echo "outer\n"; return needsTwo($a); }
outerMissing($argc);
