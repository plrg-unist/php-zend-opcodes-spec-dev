--TEST--
Function and variadic parameter documentation survives compilation
--FILE--
<?php
/** Join the "pieces".
 * See Docs\Join.
 */
function joinPieces(/** Pieces in order. */ string ...$pieces): string {
    return implode('', $pieces);
}

$function = new ReflectionFunction('joinPieces');
echo $function->getDocComment(), "\n";
echo $function->getParameters()[0]->getDocComment(), "\n";
echo joinPieces('a', 'b'), "\n";
--EXPECT--
/** Join the "pieces".
 * See Docs\Join.
 */
/** Pieces in order. */
ab
