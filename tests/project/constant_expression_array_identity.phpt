--TEST--
Deferred array constants preserve identity across reads and array casts
--FILE--
<?php
namespace Base {
    const ROW = [NAN];
}
namespace Probe {
    use const Base\ROW;
    $before = ROW;
    const ROWS = [ROW, ROW, (array) ROW, ROW + [], [] + ROW, [...ROW]];
    foreach (ROWS as $row) {
        echo (int) ($row === $before);
    }
    $after = ROWS;
    $after[0][0] = 7;
    echo "|", (int) (ROWS[0] === ROW), (int) ($after[1] === $before), "|", $after[0][0], "\n";
}
--EXPECT--
111000|11|7
