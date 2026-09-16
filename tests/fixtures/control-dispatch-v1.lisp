; #217 — canonical explicit-result dispatch witnesses.
; Each three-part clause is (query expected-datum expression).

((expr . "(cond ((quote (structural-kind pair)) (structural-kind pair) (quote hit)))")
 (expected . "hit")
 (active . t))

((expr . "(cond ((quote (structural-kind atom)) (structural-kind pair) (quote miss)) ((quote (structural-kind atom)) (structural-kind atom) (quote hit)))")
 (expected . "hit")
 (active . t))

; Empty list is data that may be matched explicitly. It is not an implicit NO.
((expr . "(cond (() () (quote empty-matched)))")
 (expected . "empty-matched")
 (active . t))

; Arbitrary non-empty data is not implicitly YES; it selects only by explicit match.
((expr . "(cond ((quote radio) antenna (quote wrong)) ((quote radio) radio (quote matched)))")
 (expected . "matched")
 (active . t))

; Exact rational binary results dispatch by explicit equality.
((expr . "(cond (1/1 1/1 (quote yes)) (0/1 0/1 (quote no)))")
 (expected . "yes")
 (active . t))

; No matching clause means no answer, not FALSE.
((expr . "(cond ((quote radio) antenna (quote wrong)))")
 (expected . "()")
 (active . t))
