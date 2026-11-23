<?php
class Snapshot {
    const VALUE = [NAN], OTHER = [NAN];
    public $value = self::VALUE;
}
function stored_value() {
    static $value = Snapshot::VALUE;
    return $value;
}
#[Sample(Snapshot::VALUE)]
function default_value($value = Snapshot::VALUE) { return $value; }
function pick($value, $left, $right) { return $value; }

$class = Snapshot::class;
$original = Snapshot::VALUE;
$attribute = (new ReflectionFunction('default_value'))->getAttributes()[0];
echo (int) ($original === $class::VALUE), (int) ($original === $class::OTHER), "\n";
echo (int) ($original === (new Snapshot)->value), (int) ($original === stored_value()), "\n";
echo (int) ($original === default_value()), (int) ($original === $attribute->getArguments()[0]), "\n";
$bound = pick(Snapshot::VALUE, right: ?, left: ?);
echo (int) ($original === $bound(0, 1)), "\n";

$copy = $original;
$copy[0] = 42;
echo (int) ($original === $copy), (int) ($original === $class::VALUE), "\n";
