; #216 — current Lisp-owned exact-Q binary decision witnesses.
; Expected semantic outcomes live here. Rust only executes expressions and
; transports actual outcomes to tests/fixtures/witness-runner.lisp.
;
; Exact rational decisions specialize to 1/1 (YES) or 0/1 (NO).
; Inexact numeric values are outside this absolute binary layer and therefore
; produce Canon 0: (), meaning no answer from this layer, never FALSE.

((expr . "(= 1/3 2/6)")
 (expected . "1")
 (active . t)
 (domain . exact-q-binary)
 (decision . yes))

((expr . "(< 2/3 3/4)")
 (expected . "1")
 (active . t)
 (domain . exact-q-binary)
 (decision . yes))

((expr . "(> 2/3 3/4)")
 (expected . "0")
 (active . t)
 (domain . exact-q-binary)
 (decision . no))

((expr . "(= 3 3)")
 (expected . "1")
 (active . t)
 (domain . exact-q-binary)
 (decision . yes))

((expr . "(< 1 2 3 4)")
 (expected . "1")
 (active . t)
 (domain . exact-q-binary)
 (decision . yes))

((expr . "(= 3.0 3.0)")
 (expected . "()")
 (active . t)
 (domain . exact-q-binary)
 (decision . no-answer))

((expr . "(< 0.5 1.0)")
 (expected . "()")
 (active . t)
 (domain . exact-q-binary)
 (decision . no-answer))
