<?php
$copy = $_ENV;
$copy['test.case'] = 0;
echo (int) (bool) $copy;
unset($copy['test.case']);
echo (int) (bool) $copy;
