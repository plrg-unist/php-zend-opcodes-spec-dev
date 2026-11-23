# Automatic cycle collection

The spec never starts cycle collection on its own. Conceptually, the collector’s candidate buffer is unbounded. An unreachable cycle can therefore continue to own a reference to a live value. Releasing the last owner of an acyclic value still frees it immediately. An explicit `gc_collect_cycles()` call requests collection of the cycles present at that point. Its return count can differ if Zend already collected some of them. Request shutdown has its own cleanup rules.

The timing is an engine choice that the opcodes alone cannot determine. Pinned Zend initially buffers 10,000 candidates and adjusts the threshold after a collection. Even two sources with identical compiled listings can leave different candidates in the buffer before the first opcode runs. [Zend collector](../../php-src/Zend/zend_gc.c#L660)

For example, suppose `$a[0]` and an unreachable cyclic array share a reference to `1`. Then `$b = $a; $b[0] = 7; echo $a[0], $b[0];` prints `77` while both owners exist. If Zend has already collected the cycle, the write separates the copied element and it prints `17`. The spec prints `77` in both cases. The regression records the exact expected stdout on each side and still compares stderr and exit status with Zend. [PHP example](../tests/project/automatic_cycle_collection.php) · [Test expectations](../tests/semantics.json)
