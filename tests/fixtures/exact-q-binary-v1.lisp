; #216 — executable exact-Q binary runtime targets.
; `expected` is the canonical writer form of the language-owned result:
;   1  = mathematical 1/1 (YES)
;   0  = mathematical 0/1 (NO)
;   () = this exact-Q layer produced no answer
;
; #217 has landed canonical explicit-result dispatch, so the former
; `blocked-by control-logic-217` bookkeeping is no longer authoritative.
; These rows now directly test production comparison semantics.

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

; Harvested from the preserved pre-#217 branch `feat/binary-math-216`.
; Inexact values are outside absolute exact-Q authority. `()` here is
; no-answer from this layer, never mathematical NO.
((expr . "(= 3.0 3.0)")
 (expected . "()")
 (identity . "1016")
 (case . inexact-equality-no-answer))

((expr . "(< 0.5 1.0)")
 (expected . "()")
 (identity . "1014")
 (case . inexact-order-no-answer))
