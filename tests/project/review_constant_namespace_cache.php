<?php
namespace { const Marker = "global"; }
namespace Review {
    $passes = 0;
    again:
    echo Marker, "|";
    if (++$passes === 2) { goto done; }
    const Marker = "local";
    goto again;
    done:
    echo Marker, "\n";
}
