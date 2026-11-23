<?php
echo "before\n";
echo match ($argc) { "1", 2 => "wrong" };
