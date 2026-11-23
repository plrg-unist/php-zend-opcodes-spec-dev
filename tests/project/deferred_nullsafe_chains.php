<?php
const NIL = null;
const NUMBER = 2;
const SKIPPED = NIL?->{MISSING}[MISSING]->{MISSING};
const REGULAR = (NIL ?? NIL?->p)->p;
const TERNARY = (NUMBER ? NIL?->p : MISSING)->p;
const NON_NULL = NUMBER?->p;
echo SKIPPED === null, REGULAR === null, TERNARY === null, NON_NULL === null;
