<?php
const PREFIX = [];
const NAME = 3;
echo "before";
const RESULT = stdClass::{(PREFIX . "x") ? NAME : "unused"};
