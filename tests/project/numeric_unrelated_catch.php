<?php
try {
    echo $argc / $argc;
} catch (TypeError $error) {
    echo "caught";
}
$bad = "bad";
echo $bad / $argc;
