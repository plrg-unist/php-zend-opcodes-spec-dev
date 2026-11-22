--TEST--
Read a class constant with a dynamic name or class
--FILE--
<?php
class Constants {
    const VALUE = 'found';
    static function read($name) { return self::{$name}; }
    static function invalid() { return self::{42}; }
}

function readConstant($class, $name) { return $class::{$name}; }

echo Constants::read('VALUE'), "\n", readConstant(Constants::class, 'class'), "\n";
try {
    Constants::invalid();
} catch (TypeError $error) {
    echo $error->getMessage(), "\n";
}
--EXPECT--
found
Constants
Cannot use value of type int as class constant name
