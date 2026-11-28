<?php
function changeTable($saveCopy) {
    global $table, $snapshot;
    unset($table['a'], $table['b']);
    if ($saveCopy) $snapshot = $table;
    $table['i'] = 9;
}

foreach ([false, true] as $saveCopy) {
    $table = ['a'=>1, 'b'=>2, 'c'=>3, 'd'=>4, 'e'=>5, 'f'=>6, 'g'=>7, 'h'=>8];
    foreach ($table as $first => &$outer) {
        echo $first;
        if ($first === 'a') {
            echo '(';
            foreach ($table as $second => &$inner) {
                echo $second;
                if ($second === 'h') changeTable($saveCopy);
            }
            unset($inner);
            echo ')';
        }
    }
    unset($outer);
    echo ':', $table['i'];
    if ($saveCopy) echo ':', count($snapshot), ':', $snapshot['c'];
    echo "\n";
}
