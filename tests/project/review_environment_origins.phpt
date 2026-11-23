--TEST--
CLI server and environment arrays share imported entries across comparisons and unions
--FILE--
<?php
$env = $_ENV;
$server = $_SERVER;
echo (int) ($env == $server), (int) ($env === $server), "|";
$union = $server + $env;
echo (int) ($union === $server), "|";
$env["test.extra"] = 7;
$union = $server + $env;
echo $union["test.extra"], ":", $union["argc"], "|";
unset($env["test.extra"]);
foreach (["PHP_SELF", "SCRIPT_NAME", "SCRIPT_FILENAME", "PATH_TRANSLATED", "DOCUMENT_ROOT",
          "REQUEST_TIME_FLOAT", "REQUEST_TIME", "argv", "argc"] as $name) {
    unset($env[$name], $server[$name]);
}
echo (int) ($env === $server), ":", $env <=> $server, "|";
$left = $env + $server;
$right = $server + $env;
echo (int) ($left === $right), (int) ($left === $env), "\n";
--EXPECT--
00|1|7:1|1:0|11
