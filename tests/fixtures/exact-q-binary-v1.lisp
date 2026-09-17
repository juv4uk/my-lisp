; #216 — executable exact-Q binary targets.
; `expected` is the canonical writer form of the exact rational result:
; 1 means mathematical 1/1 (YES), 0 means mathematical 0/1 (NO).
;
; RED was first proven in CI #2224 while #217 still blocked runtime activation.
; #217 and #229 have now landed explicit result dispatch and layered Canon laws,
; so these rows are active runtime requirements rather than blocked design debt.
;
; Values outside the exact rational domain do not receive a binary answer from
; this layer. Inexact numeric input therefore yields Canon 0: (), never FALSE.

((expr . "(= 1/3 2/6)")
 (expected . "1")
 (identity . "1016")
 (case . rational-equality-yes))

((expr . "(< 2/3 3/4)")
 (expected . "1")
 (identity . "1014")
 (case . rational-less-yes))

((expr . "(> 2/3 3/4)")
 (expected . "0")
 (identity . "1015")
 (case . rational-greater-no))

((expr . "(<= 1/3 1/3 2/3)")
 (expected . "1")
 (identity . "1017")
 (case . rational-nondecreasing-yes))

((expr . "(>= 3/4 2/3 2/3)")
 (expected . "1")
 (identity . "1018")
 (case . rational-nonincreasing-yes))

((expr . "(= 3.0 3.0)")
 (expected . "()")
 (identity . "1016")
 (case . inexact-equality-no-answer))

((expr . "(< 0.5 1.0)")
 (expected . "()")
 (identity . "1014")
 (case . inexact-less-no-answer))
