; #216 — executable exact-Q binary targets.
; `expected` is the canonical writer form of the exact rational result:
; 1 means mathematical 1/1 (YES), 0 means mathematical 0/1 (NO).
; Current runtime should RED here because comparisons still return t/().

((expr . "(= 1/3 2/6)")
 (expected . "1")
 (active . t)
 (identity . "1016")
 (case . rational-equality-yes))

((expr . "(< 2/3 3/4)")
 (expected . "1")
 (active . t)
 (identity . "1014")
 (case . rational-less-yes))

((expr . "(> 2/3 3/4)")
 (expected . "0")
 (active . t)
 (identity . "1015")
 (case . rational-greater-no))

((expr . "(<= 1/3 1/3 2/3)")
 (expected . "1")
 (active . t)
 (identity . "1017")
 (case . rational-nondecreasing-yes))

((expr . "(>= 3/4 2/3 2/3)")
 (expected . "1")
 (active . t)
 (identity . "1018")
 (case . rational-nonincreasing-yes))
