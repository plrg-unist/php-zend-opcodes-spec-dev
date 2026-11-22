<?php
class KeyLog extends ArrayObject {
    public function offsetGet(mixed $key): mixed {
        return get_debug_type($key);
    }
}

$object = new KeyLog();
var_dump($object["1"], $object[1], $object["01"]);
