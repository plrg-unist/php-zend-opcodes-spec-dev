<?php
echo "<", $missing, ">", $missing, "\n";
echo gettype($missing), ":", is_null($missing), "\n";
echo "len=", strlen($other), "\n";
echo "copy=", ($copy = $missing), ":", gettype($copy), "\n";
echo "xor=", ($first xor $second), "\n";
echo "concat=", $first . $second, "\n";
echo "append=", ($left .= $right), ":", gettype($left), "\n";
echo "self=", ($self .= $self), ":", gettype($self), "\n";
echo "increment=", ++$uninitialized, ":", $uninitialized, "\n";
is_int($discarded);
(void) ($first . $second);
if ($missing) { echo "wrong"; } else { echo "branch"; }
if ($discarded) {}
if ($copy) {}
echo ${"name\0\xff"};
$nan = NAN;
echo ":", ($nan xor $absent), ":", ($absent xor $nan), "\n";
