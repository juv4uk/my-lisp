; #216 — executable exact-Q binary targets.
; `expected` is the canonical writer form of the exact rational result:
; 1 means mathematical 1/1 (YES), 0 means mathematical 0/1 (NO).
;
; RED was proven in CI #2224: the Lisp-owned contract self-check passed while
; the current runtime returned historical `t` for (= 1/3 2/6), where this
; corpus requires the exact rational decision 1/1 (canonical write "1").
;
; These result rows now stay executable design targets but are blocked by
; #217. Switching comparison results before canonical control stops consuming
; generic truthiness would break old cond/bootstrap semantics.

((expr . "(= 1/3 2/6)")
 (expected . "1")
 (blocked-by . control-logic-217)
 (identity . "1016")
 (case . rational-equality-yes))

((expr . "(< 2/3 3/4)")
 (expected . "1")
 (blocked-by . control-logic-217)
 (identity . "1014")
 (case . rational-less-yes))

((expr . "(> 2/3 3/4)")
 (expected . "0")
 (blocked-by . control-logic-217)
 (identity . "1015")
 (case . rational-greater-no))

((expr . "(<= 1/3 1/3 2/3)")
 (expected . "1")
 (blocked-by . control-logic-217)
 (identity . "1017")
 (case . rational-nondecreasing-yes))

((expr . "(>= 3/4 2/3 2/3)")
 (expected . "1")
 (blocked-by . control-logic-217)
 (identity . "1018")
 (case . rational-nonincreasing-yes))
