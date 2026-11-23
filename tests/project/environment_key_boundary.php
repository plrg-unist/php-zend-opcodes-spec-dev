<?php
$copy = $_ENV;
echo (int) is_array($copy), (int) isset($copy['impossible.name']);
echo (int) isset($copy['REQUEST_TIME']);
