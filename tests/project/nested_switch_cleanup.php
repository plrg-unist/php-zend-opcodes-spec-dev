<?php
$x = 2;
$y = 3;
switch ($x . $y) {
    default:
        switch ($y . $x) {
            default:
                echo "ok";
                return 2;
        }
}
