<?php
#[LocalMark("global")]
const BASE = 3;

class Bounds {
    public const STEP = 2;
}

#[LocalMark(BASE, label: Bounds::STEP)]
function calculate(
    #[LocalParam] (Countable&Iterator)|string $tag = "ok",
    array $values = [BASE + Bounds::STEP, BASE => Bounds::STEP],
): array {
    return [$tag, $values];
}

$function = new ReflectionFunction("calculate");
$parameter = $function->getParameters()[0];
$constant = (new ReflectionConstant("BASE"))->getAttributes()[0];
var_dump(calculate(), $function->getAttributes()[0]->getArguments(),
    (string) $parameter->getType(), $parameter->getAttributes()[0]->getName(),
    $constant->getName(), $constant->getArguments());
