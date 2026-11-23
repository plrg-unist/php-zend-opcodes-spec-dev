--TEST--
Named constants preserve the case of the final component and protect arrays from writes to fetched copies
--FILE--
<?php
namespace {
    const Root = 7;
}
namespace Sample\Nested {
    const A = [Root, [Root + 1]];
    const a = 19;
    $copy = A;
    $copy[1][0] = 30;
    echo \sAMPLe\neSTED\A[1][0], ":", $copy[1][0], ":", a, ":", A[0], "\n";
    echo (int)\defined("Root"), ":", (int)\defined("root"), "\n";
}
--EXPECT--
8:30:19:7
1:0
