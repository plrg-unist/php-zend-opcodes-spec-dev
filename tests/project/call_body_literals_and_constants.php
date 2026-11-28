<?php
namespace { const Marker = "global"; }
namespace Scope {
    function readOne() { return Marker; }
    function readTwo() { return Marker; }
    function firstTable($n) { if ($n) { echo 'f'; return [\NAN]; } return []; }
    function secondTable($n) { if ($n) { echo 's'; return [\NAN]; } return []; }

    echo \Scope\readOne(), "|";
    const Marker = "local";
    echo \Scope\readOne(), "|", \Scope\readTwo(), "\n";
    $first = \Scope\firstTable($argc);
    $same = \Scope\firstTable($argc);
    $other = \Scope\secondTable($argc);
    echo (int) ($first === $same), ":", (int) ($first === $other), "\n";
}
