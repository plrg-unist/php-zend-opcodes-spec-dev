<?php
const LIST_VALUE = [1];
const TEXT = "4tail";
const BAD = (LIST_VALUE . LIST_VALUE) . (TEXT + 1);
echo BAD, "|";
const FAIL = LIST_VALUE . (MISSING + "never");
echo "unreachable";
