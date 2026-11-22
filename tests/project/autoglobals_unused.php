<?php
foreach (['_ENV', '_REQUEST'] as $name) {
    var_dump(isset($GLOBALS[$name]));
}
