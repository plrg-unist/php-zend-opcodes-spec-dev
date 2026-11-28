--TEST--
Deeply nested compiled units survive export, loading, and structural validation
--FILE--
<?php
// Each parameter default is present in the compiled unit even though dormant() is never called.
function dormant(
    // 65 closures nested through their parameter defaults.
    $closure =
        static function ($v = static function ($v = static function ($v =
        static function ($v = static function ($v = static function ($v =
        static function ($v = static function ($v = static function ($v =
        static function ($v = static function ($v = static function ($v =
        static function ($v = static function ($v = static function ($v =
        static function ($v = static function ($v = static function ($v =
        static function ($v = static function ($v = static function ($v =
        static function ($v = static function ($v = static function ($v =
        static function ($v = static function ($v = static function ($v =
        static function ($v = static function ($v = static function ($v =
        static function ($v = static function ($v = static function ($v =
        static function ($v = static function ($v = static function ($v =
        static function ($v = static function ($v = static function ($v =
        static function ($v = static function ($v = static function ($v =
        static function ($v = static function ($v = static function ($v =
        static function ($v = static function ($v = static function ($v =
        static function ($v = static function ($v = static function ($v =
        static function ($v = static function ($v = static function ($v =
        static function ($v = static function ($v = static function ($v =
        static function ($v = static function ($v = static function ($v =
        static function ($v = static function ($v = static function ($v =
        static function ($v = static function ($v =
        null
        ) {} ) {} ) {} ) {} ) {} ) {} ) {} ) {} ) {} ) {} ) {} ) {}
        ) {} ) {} ) {} ) {} ) {} ) {} ) {} ) {} ) {} ) {} ) {} ) {}
        ) {} ) {} ) {} ) {} ) {} ) {} ) {} ) {} ) {} ) {} ) {} ) {}
        ) {} ) {} ) {} ) {} ) {} ) {} ) {} ) {} ) {} ) {} ) {} ) {}
        ) {} ) {} ) {} ) {} ) {} ) {} ) {} ) {} ) {} ) {} ) {} ) {}
        ) {} ) {} ) {} ) {} ) {},
    // 257 additions remain deferred because the constant is unresolved.
    $number = NESTING_BASE
        + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1
        + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1
        + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1
        + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1
        + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1
        + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1
        + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1
        + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1
        + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1
        + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1
        + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1
        + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1
        + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1
        + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1
        + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1
        + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1
        + 1,
    // 257 literal array layers.
    $array =
        [[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[
        [[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[
        [[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[
        [[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[
        [[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[
        [[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[
        [[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[
        [[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[
        [
        9
        ]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]
        ]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]
        ]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]
        ]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]
        ]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]
        ]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]
        ]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]
        ]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]
        ]
) {
    // Each arrow function contributes one nested child body.
    return
        fn() => fn() => fn() => fn() => fn() => fn() => fn() => fn() => fn() =>
        fn() => fn() => fn() => fn() => fn() => fn() => fn() => fn() => fn() =>
        fn() => fn() => fn() => fn() => fn() => fn() => fn() => fn() => fn() =>
        fn() => fn() => fn() => fn() => fn() => fn() => fn() => fn() => fn() =>
        fn() => fn() => fn() => fn() => fn() => fn() => fn() => fn() => fn() =>
        fn() => fn() => fn() => fn() => fn() => fn() => fn() => fn() => fn() =>
        fn() => fn() => fn() => fn() => fn() => fn() => fn() => fn() => fn() =>
        fn() => fn() => fn() => fn() => fn() => fn() => fn() => fn() => fn() =>
        fn() => fn() => fn() => fn() => fn() => fn() => fn() => fn() => fn() =>
        fn() => fn() => fn() => fn() => fn() => fn() => fn() => fn() => fn() =>
        fn() => fn() => fn() => fn() => fn() => fn() => fn() => fn() => fn() =>
        fn() => fn() => fn() => fn() => fn() => fn() => fn() => fn() => fn() =>
        fn() => fn() => fn() => fn() => fn() => fn() => fn() => fn() => fn() =>
        fn() => fn() => fn() => fn() => fn() => fn() => fn() => fn() => fn() =>
        fn() => fn() => fn() => fn() => fn() => fn() => fn() => fn() => fn() =>
        fn() => fn() => fn() => fn() => fn() => fn() => fn() => fn() => fn() =>
        fn() => fn() => fn() => fn() => fn() => fn() => fn() => fn() => fn() =>
        fn() => fn() => fn() => fn() => fn() => fn() => fn() => fn() => fn() =>
        fn() => fn() => fn() => fn() => fn() => fn() => fn() => fn() => fn() =>
        fn() => fn() => fn() => fn() => fn() => fn() => fn() => fn() => fn() =>
        fn() => fn() => fn() => fn() => fn() => fn() => fn() => fn() => fn() =>
        fn() => fn() => fn() => fn() => fn() => fn() => fn() => fn() => fn() =>
        fn() => fn() => fn() => fn() => fn() => fn() => fn() => fn() => fn() =>
        fn() => fn() => fn() => fn() => fn() => fn() => fn() => fn() => fn() =>
        fn() => fn() => fn() => fn() => fn() => fn() => fn() => fn() => fn() =>
        fn() => fn() => fn() => fn() => fn() => fn() => fn() => fn() => fn() =>
        fn() => fn() => fn() => fn() => fn() => fn() => fn() => fn() => fn() =>
        fn() => fn() => fn() => fn() => fn() => fn() => fn() => fn() => fn() =>
        fn() => fn() => fn() => fn() => fn() =>
        9;
}
echo "ok\n";
--EXPECT--
ok
