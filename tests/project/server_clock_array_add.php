<?php
$seconds =& $_SERVER['REQUEST_TIME'];
$fraction =& $_SERVER['REQUEST_TIME_FLOAT'];
$ambient =& $_ENV['ZEND_COMPARE_FIELD'];
echo (int) ([$seconds] === [$fraction]), '|', (int) is_int($seconds ?? 0.5), '|';
echo NAN <=> $seconds, $seconds <=> NAN, NAN <=> $fraction, $fraction <=> NAN, '|';
echo (int) ($fraction !== NAN), (int) (NAN !== $fraction), '|';
echo (int) ([$fraction] == [NAN]), (int) ([false] === [$fraction]), '|';
echo (int) ($ambient === 4), (int) (4 === $ambient), (int) ([$ambient] === [$seconds]), '|';
echo (int) ($fraction === $fraction), (int) ([$fraction] == [$fraction]), $fraction <=> $fraction, '|';
echo (int) ($ambient === $_ENV['ZEND_COMPARE_FIELD']), (int) ([$ambient] == [$_ENV['ZEND_COMPARE_FIELD']]), '|';
$wrapped = (array) $seconds;
echo (int) is_int($wrapped[0]), '|';
foreach ($seconds as $item) { echo 'entered'; }
foreach ($seconds as &$item) { echo 'entered'; }
['missing' => $item] = $fraction;
echo (int) ($item === null), "\n";
echo [] + $seconds;
