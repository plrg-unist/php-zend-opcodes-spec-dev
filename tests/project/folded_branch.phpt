--TEST--
The default optimizer removes a constant conditional jump
--FILE--
<?php
function answer() {
    $value = 40;
    if (false) {
        return 0;
    }
    return $value + 2;
}

$listing = ini_get('zend_opcodes.output');
$profile = getenv('ZEND_OPCODES_TEST_PROFILE');
if ($listing && $profile !== false) {
    $hasBranch = preg_match('/^ *\d+: JMPZ lit\[/m', file_get_contents($listing));
    if ($hasBranch !== (int) ($profile === 'ordinary')) {
        echo "unexpected conditional jump in $profile\n";
    }
}
echo answer(), "\n";
--EXPECT--
42
