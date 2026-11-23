<?php
echo "C";
try {
    echo count($argc);
} catch (TypeError $error) {
    echo "caught\n";
}
