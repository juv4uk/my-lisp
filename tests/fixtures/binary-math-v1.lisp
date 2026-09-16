; #216 — Lisp-owned exact-Q binary decision corpus.
; The expected result domain is data here; Rust transports only observations.
;
; Exact-Q decisions must produce exact rational 0/1 or 1/1 values. The
; ordinary printer renders denominator-1 rationals as "0" / "1". Inexact
; operands are outside this layer and therefore produce Canon 0: ().

((expr . "(= 1/3 2/6)")
 (expected-kind . rational)
 (expected-render . "1")
 (decision . yes))

((expr . "(< 2/3 3/4)")
 (expected-kind . rational)
 (expected-render . "1")
 (decision . yes))

((expr . "(> 2/3 3/4)")
 (expected-kind . rational)
 (expected-render . "0")
 (decision . no))

((expr . "(= 3 3)")
 (expected-kind . rational)
 (expected-render . "1")
 (decision . yes))

((expr . "(< 1 2 3 4)")
 (expected-kind . rational)
 (expected-render . "1")
 (decision . yes))

((expr . "(= 3.0 3.0)")
 (expected-kind . empty-list)
 (expected-render . "()")
 (decision . no-answer))

((expr . "(< 0.5 1.0)")
 (expected-kind . empty-list)
 (expected-render . "()")
 (decision . no-answer))
