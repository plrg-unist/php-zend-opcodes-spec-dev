--TEST--
CLI autoglobals, attached CVs, and newly created names keep their symbol-table order
--FILE--
<?php
if ($argc > 1) {
    echo $_REQUEST, $_SERVER, $_ENV, $second, $first;
}
foreach ($GLOBALS as $key => $value) {
    echo $key, ",";
}
echo "\n";
$name = "dynamic";
$$name = 1;
$first = 2;
$second = 3;
foreach ($GLOBALS as $key => $value) {
    echo $key, ",";
}
echo "\n";
unset($second, $$name);
$name = "extra";
$$name = 4;
$second = 5;
$name = "dynamic";
$$name = 6;
foreach ($GLOBALS as $key => $value) {
    echo $key, ",";
}
echo "\n", count($_REQUEST), ":", is_array($_ENV), "\n";
--EXPECT--
argv,argc,_GET,_POST,_COOKIE,_FILES,_SERVER,_REQUEST,_ENV,
argv,argc,_GET,_POST,_COOKIE,_FILES,_SERVER,_REQUEST,_ENV,second,first,value,key,name,dynamic,
argv,argc,_GET,_POST,_COOKIE,_FILES,_SERVER,_REQUEST,_ENV,second,first,value,key,name,extra,dynamic,
0:1
