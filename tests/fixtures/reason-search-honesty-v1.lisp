; #219 — search-scope honesty witness.
; A returned finite-snapshot search may report what it actually established:
; no proof on either side under that search. It must not silently elevate that
; observation to an absolute `(unknown subject)` claim.

((expr . "(reason-observe (quote (parent bob alice)) (quote (((parent alice bob)))))")
 (expected . "(unknown (not-proved-under-search (parent bob alice) (reason-search finite-snapshot exhausted)))")
 (active . t)
 (law . search-unknown-must-name-its-scope))
