--TEST--
Snapshots preserve dynamic symbol order and convert canonical integer keys
--FILE--
<?php
$names = ["0", "00", "-0", "-1", "+1", "", "a\0b", "9223372036854775807", "9223372036854775808", "-9223372036854775808", "-9223372036854775809"];
foreach ($names as $i => $name) {
    $$name = 60 + $i;
}
$before = $GLOBALS;
$name = "0";
unset($$name);
$$name = 90;
$after = $GLOBALS;
foreach ([$before, $after] as $snapshot) {
    foreach ($snapshot as $key => $value) {
        if (is_int($value) && $value >= 60 && $value <= 90) {
            echo is_int($key) ? "i" : "s", "(", $key === "a\0b" ? "binary" : $key, ")=", $value, "|";
        }
    }
    echo "\n";
}
echo $GLOBALS[0], ":", $GLOBALS["00"], ":", $before[0], ":", $after[0], "\n";
--EXPECT--
i(0)=60|s(00)=61|s(-0)=62|i(-1)=63|s(+1)=64|s()=65|s(binary)=66|i(9223372036854775807)=67|s(9223372036854775808)=68|i(-9223372036854775808)=69|s(-9223372036854775809)=70|
s(00)=61|s(-0)=62|i(-1)=63|s(+1)=64|s()=65|s(binary)=66|i(9223372036854775807)=67|s(9223372036854775808)=68|i(-9223372036854775808)=69|s(-9223372036854775809)=70|i(0)=90|
90:61:60:90
