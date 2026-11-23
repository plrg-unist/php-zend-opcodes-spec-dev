<?php
$subjects = 0;
$tests = -1;
switch ((string) ++$subjects) {
    case ++$tests: echo "wrong"; break;
    case ++$tests: echo "loose"; break;
    case $never / 0: echo "wrong"; break;
    default: echo "wrong";
}
$result = match ((string) ++$subjects) {
    ++$tests => "wrong",
    (string) $tests => "strict",
    $never / 0 => "wrong",
    default => "wrong",
};
echo ":", $result, ":", $subjects, ":", $tests, "\n";
