<?php
class DeferredScope {
    public const OFFSET = 8;
    public $callback = DeferredScope::plus(...);

    public static function plus($value) {
        return self::OFFSET + $value;
    }

    public function withDefault($callback = static function ($value = 2) {
        return self::OFFSET + $value;
    }) {
        return $callback();
    }
}

$object = new DeferredScope();
var_dump($object->withDefault(), ($object->callback)(4));
