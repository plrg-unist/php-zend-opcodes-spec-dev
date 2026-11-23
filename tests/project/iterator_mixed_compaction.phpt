--TEST--
Live array iterators at a deleted bucket and past the end survive compaction and copying
--FILE--
<?php
foreach ([false, true] as $copyBeforeInsert) {
    $table = ["a"=>1, "b"=>2, "c"=>3, "d"=>4, "e"=>5, "f"=>6, "g"=>7, "h"=>8];
    foreach ($table as $first => &$outer) {
        echo $first;
        if ($first === "a") {
            echo "(";
            foreach ($table as $second => &$inner) {
                echo $second;
                if ($second === "h") {
                    unset($table["a"], $table["b"]);
                    if ($copyBeforeInsert) {
                        $snapshot = $table;
                    }
                    $table["i"] = 9;
                }
            }
            unset($inner);
            echo ")";
        }
    }
    unset($outer);
    echo ":", $table["i"];
    if ($copyBeforeInsert) {
        echo ":", count($snapshot), ":", $snapshot["c"];
    }
    echo "\n";
}
--EXPECT--
a(abcdefghi)cdefghi:9
a(abcdefghi)cdefghi:9:6:3
