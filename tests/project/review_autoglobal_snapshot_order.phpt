--TEST--
Lazy globals keep their registration order and their values separate across snapshots
--FILE--
<?php
if (!$argc) {
    $_REQUEST = [];
    $_ENV = [];
}
foreach ($GLOBALS as $key => $value) {
    echo $key, "|";
}
echo "\n";
$_GET["field"] = 4;
$_REQUEST["field"] = 5;
$_ENV["test.key"] = [6];
$snapshot = $GLOBALS;
$snapshot["_ENV"]["test.key"][0] = 7;
unset($_REQUEST);
echo $_GET["field"], $snapshot["_REQUEST"]["field"], (int) isset($_REQUEST),
    $_ENV["test.key"][0], $snapshot["_ENV"]["test.key"][0], "\n";
--EXPECT--
argv|argc|_GET|_POST|_COOKIE|_FILES|_SERVER|_REQUEST|_ENV|
45067
