; #216 — executable exact-Q binary runtime targets.
; First runtime activation slice after #217/#229: primitive <, >, = only.
;
; `expected` is the canonical writer form of the language-owned result:
;   1  = mathematical 1/1 (YES)
;   0  = mathematical 0/1 (NO)
;   () = this exact-Q layer produced no answer
;
; <= and >= remain ratified by contracts/exact-q-binary-contract.lisp but are
; intentionally deferred to the next slice: their current Lisp definitions
; still contain migration-era two-part cond consumers and must be migrated
; explicitly rather than teaching generic control that numeric 0 means false.

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
