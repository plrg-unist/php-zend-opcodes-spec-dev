<?php
declare(strict_types=1);

const N = 3;
enum Choice { case First; }

class Box {
    public const INDEX = 0;
    public function __construct(public int $n) {}
    public static function identity($n) { return $n; }
}

trait ScopeNames {
    public function names($owner = __CLASS__, $type = self::class) {
        return [$owner, $type];
    }
}

class Home { use ScopeNames; }

#[Marker(N, key: Box::INDEX)]
function expressions(
    #[Slot] $values = [
        +N, -N, !N, (string) N, N + 2, N > N, N >= N, N && N, N || N, N ?? 5, N ?: 5,
        [N][Box::INDEX], Choice::First->name, Choice::First?->name, ...[N],
    ],
    $box = new Box(n: N),
    $closure = static function ($inner = static function () { return N; }) { return $inner(); },
    $function = abs(...),
    $method = Box::identity(?),
) {
    return [$values, $box->n, $closure(), $function(-4), $method(5)];
}

$attributes = (new ReflectionFunction("expressions"))->getAttributes();
var_dump((new Home())->names(), expressions(), $attributes[0]->getArguments());
