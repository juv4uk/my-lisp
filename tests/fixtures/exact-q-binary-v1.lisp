; #216 — executable exact-Q binary targets.
; `expected` is the canonical writer form of the exact rational result:
; 1 means mathematical 1/1 (YES), 0 means mathematical 0/1 (NO).
;
; RED was first proven in CI #2224 while #217 still blocked runtime activation.
; #217 and #229 have now landed explicit result dispatch and layered Canon laws,
; so these rows are active runtime requirements rather than blocked design debt.
;
; Values outside the exact rational domain do not receive a binary answer from
; this layer. Decimal source literals are exact rationals in my-lisp; JSON
; decimal tokens are an existing explicit inexact boundary and therefore make
; the non-applicability witnesses below real rather than syntactic guesses.

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

; #295 — strict comparison laws preserved after retiring the legacy Rust
; truth-sentinel test. The mathematical answer stays in the existing exact-Q
; result domain; no new host-side semantic oracle is introduced.
((expr . "(< 1 2 3)")
 (expected . "1")
 (identity . "1014")
 (case . strict-chain-yes))

((expr . "(< 1 3 2)")
 (expected . "0")
 (identity . "1014")
 (case . strict-chain-no))

((expr . "(< 5)")
 (expected . "1")
 (identity . "1014")
 (case . strict-single-argument-vacuous-yes))

((expr . "(= (json-parse \"3.0\") (json-parse \"3.0\"))")
 (expected . "()")
 (identity . "1016")
 (case . inexact-equality-no-answer))

((expr . "(< (json-parse \"0.5\") (json-parse \"1.0\"))")
 (expected . "()")
 (identity . "1014")
 (case . inexact-less-no-answer))


; #216 acceptance — denominator-1 source spellings must agree with the
; mathematical exact-Q value and with the canonical writer representation.
; These rows distinguish source spelling from semantic value: 0/1 and 1/1
; are admitted exact rationals, while the canonical writer compacts them to
; 0 and 1 respectively.
((expr . "(= 0 0/1)")
 (expected . "1")
 (identity . "1016")
 (case . zero-and-zero-over-one-are-the-same-exact-q-value))

((expr . "(= 1 1/1)")
 (expected . "1")
 (identity . "1016")
 (case . one-and-one-over-one-are-the-same-exact-q-value))

((expr . "(list 0 0/1 1 1/1)")
 (expected . "(0 0 1 1)")
 (case . denominator-one-source-spellings-write-canonically))
