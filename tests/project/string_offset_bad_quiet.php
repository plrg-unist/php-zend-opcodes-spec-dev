<?php
$s = "A" . $argc;
echo $s["not a number"] ?? "missing", "|", $s[[]] ?? "missing";
