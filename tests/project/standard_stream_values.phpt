--TEST--
The three CLI handles retain their type, identity and numeric and string conversions
--FILE--
<?php
foreach ([STDIN, STDOUT, STDERR] as $r) {
    echo gettype($r), '/', (int) is_resource($r), (int) is_scalar($r), (int) (bool) $r,
         '/', (int) $r, '/', (float) $r, '/', $r, '/', ((array) $r)[0] === $r, "\n";
}
$r = STDOUT;
foreach ([null, false, true, [], [0], 0, 2, 3, 2.5, NAN, '2', ' 2.0tail', 'word', STDIN, STDOUT, STDERR] as $other) {
    echo $r <=> $other, '/', $other <=> $r, '/', (int) ($r === $other), '|';
}
echo "\n", (int) ([STDOUT] === [$r]), (int) ([STDIN] == [1]), (int) ([STDERR] === [3]), "\n";
echo (int) in_array($r, ['word', '2x']), (int) in_array($r, [2, 'word'], true), '/';
$ref =& $r;
echo (int) in_array($ref, ['word', '2x']), (int) in_array($ref, [2, 'word'], true), "\n";
$s = 'abcd';
echo (int) isset($s[$r]), (int) empty($s[$r]), (int) isset($r[0]), (int) empty($r[0]), "\n";
--EXPECT--
resource/101/1/1/Resource id #1/1
resource/101/2/2/Resource id #2/1
resource/101/3/3/Resource id #3/1
1/-1/0|1/-1/0|0/0/0|-1/1/0|-1/1/0|1/-1/0|0/0/0|-1/1/0|-1/1/0|1/1/0|0/0/0|0/0/0|1/-1/0|1/-1/0|0/0/1|-1/1/0|
110
10/10
0101
