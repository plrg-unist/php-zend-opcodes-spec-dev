<?php
echo $argc, ":", count($argv), ":", is_array($argv), "\n";
echo $argv[0], "\n", __FILE__, "\n";
echo $argv[1] ?? "missing", "\n";
