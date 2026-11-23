<?php
const TEXT = "abc";
const INVALID_OFFSET = [];
echo "offset\n";
const NO_OFFSET = TEXT[INVALID_OFFSET] ?? "fallback";
