<?php
echo __FILE__, "\n", $_SERVER["SCRIPT_FILENAME"], "\n", $argv[0], "\n";
fwrite(STDERR, "message\0\n");
exit(7);
