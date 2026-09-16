; #225 — current executable mathematical-result anchors.
; These are arithmetic results, not propositions and never logical statuses.

((expr . "(+ 1/3 1/6)")
 (expected . "1/2")
 (active . t)
 (case . exact-rational-sum))

((expr . "(/ 1 8)")
 (expected . "1/8")
 (active . t)
 (case . exact-rational-division))

((expr . "(+ 2 3)")
 (expected . "5")
 (active . t)
 (case . exact-integer-result))

((expr . "(+ 0.1 0.2)")
 (expected . "3/10")
 (active . t)
 (case . exact-decimal-source-result))
