--TEST--
A failed match unwinds pending calls without disturbing a sibling branch or catch
--FILE--
<?php
function wrapValue($value) { echo "wrapped $value\n"; return $value; }

function conditionalCall($fail) {
    try {
        var_dump($fail ? wrapValue(match (true) { 1, 2, 3, 4, 5 => 'never' }) : wrapValue(7));
    } catch (UnhandledMatchError $error) {
        echo "caught\n";
    }
}

function finalCall() { var_dump(match (true) { 1, 2, 3, 4, 5 => 'never' }); }

class MagicCall {
    private static function hidden(): never { throw new Exception(); }
    public static function __callStatic($name, $args) { return $name; }
}

conditionalCall(false);
conditionalCall(true);
try {
    finalCall();
} catch (UnhandledMatchError $error) {
    echo "caught final\n";
}
var_dump(MagicCall::hidden());
--EXPECT--
wrapped 7
int(7)
caught
caught final
string(6) "hidden"
