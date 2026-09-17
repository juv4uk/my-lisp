; #219/#244 — reasoning observation honesty witness.
; Merely failing to derive either side does not positively establish the
; epistemic classification `unknown`. Without a named completeness/search
; contract that justifies that stronger claim, the answer remains Canon 0.
;
; #369 preserves the public invalid-goal result for a non-symbol goal head
; before `result-goal?` migrates away from the historical t/() shape of
; `symbol?`. The law is the reasoning outcome, not the old predicate sentinel.

((expr . "(let ((rules (quote (((parent alice bob)))))) (list (reason-observe (quote (parent bob alice)) rules) (reason-observe (quote (42 payload)) rules)))")
 (expected . "(() (invalid invalid-goal (42 payload)))")
 (active . t)
 (law . no-evidence-and-invalid-goal-honesty))
