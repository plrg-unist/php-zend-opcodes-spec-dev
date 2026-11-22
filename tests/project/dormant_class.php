<?php
$enabled = false;
if ($enabled) {
    class DormantClass {}
    class DormantSubclass extends UnavailableParent {}
}
echo "main";
