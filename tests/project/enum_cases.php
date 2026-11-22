<?php
namespace Signals;

define(__NAMESPACE__ . "\\BaseBit", 8);

interface Labelled {
    public function label(): string;
}

enum Light: string implements Labelled {
    case Stop = "R";
    public const First = self::Stop;
    case Go = "G";

    public function label(): string {
        return $this->name . $this->value;
    }
}

enum Bit: int {
    case Low = 1;
    #[Marker("high")]
    case High = BaseBit + 1;
}

enum Answer {
    case Yes;
    case No;
}

foreach (Light::cases() as $light) {
    echo $light->label(), "\n";
}
var_dump(Light::First === Light::from("R"), Light::tryFrom("X"));
echo Bit::from(9)->name, " ", Answer::cases()[1]->name, "\n";
$case = new \ReflectionEnumUnitCase(Bit::class, "High");
echo $case->getAttributes()[0]->getArguments()[0], "\n";
