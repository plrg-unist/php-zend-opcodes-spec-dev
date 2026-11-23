--TEST--
Finite reads, mutations and identity checks on self and mutually recursive arrays
--FILE--
<?php
$a = ["n" => $argc];
$a["link"] =& $a;
$root = $a;
unset($a);
$copy = $root;
$copy["n"] = 8;
$copy["link"]["n"] = 9;
echo $root["n"], $copy["n"], $root["link"]["n"], $copy["link"]["n"], "\n";
echo (int) ($root === $root), (int) ($root === $copy), (int) ($root["link"] === $copy["link"]), "\n";

$l = ["v" => 1];
$r = ["v" => 2];
$l["p"] =& $r;
$r["p"] =& $l;
$c = $l;
$c["v"] = 8;
$c["p"]["p"]["v"] = 9;
echo $l["v"], $r["v"], $c["v"], $c["p"]["p"]["v"], "\n";

$n = NAN;
$u = [&$n];
$w = [&$n];
echo (int) ($u === $u), (int) ($u === $w), (int) ($u == $w);
$n = 4;
echo (int) ($u === $w), (int) ($u == $w), "\n";
--EXPECT--
1899
101
9289
10011
