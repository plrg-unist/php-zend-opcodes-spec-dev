<?php
trait Upper {
    public function word(): string {
        return "UP";
    }
}

trait Lower {
    public function word(): string {
        return "down";
    }
}

class Dictionary {
    use Upper, Lower {
        Upper::word insteadof Lower;
        Lower::word as alternate;
    }

    public string $label {
        get => $this->word() . "/" . $this->alternate();
    }
}

echo (new Dictionary())->label, "\n";
