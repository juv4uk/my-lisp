; #216 — executable exact-Q binary runtime targets.
; First runtime activation slice after #217/#229: primitive <, >, = only.
;
; `expected` is the canonical writer form of the language-owned result:
;   1 = mathematical 1/1 (YES)
;   0 = mathematical 0/1 (NO)
;
; IMPORTANT: every finite decimal/scientific source literal in current my-lisp
; is exact by axiom S1. Therefore 3.0, 0.5, 1.0 are exact rationals here, not
; approximations. Runtime-created `Exactness::Inexact` values remain outside
; exact-Q authority and must yield Canon 0, but source syntax cannot currently
; construct such a value; that negative runtime witness waits for the ratified
; explicit inexact constructor/boundary instead of fabricating one in a host test.
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

; Decimal spelling does not imply inexactness in my-lisp.
((expr . "(= 3.0 3.0)")
 (expected . "1")
 (identity . "1016")
 (case . exact-decimal-equality-yes))

((expr . "(< 0.5 1.0)")
 (expected . "1")
 (identity . "1014")
 (case . exact-decimal-order-yes))
