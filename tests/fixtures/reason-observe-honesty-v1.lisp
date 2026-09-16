; #219/#244 — reasoning observation honesty witness.
; Merely failing to derive either side does not positively establish the
; epistemic classification `unknown`. Without a named completeness/search
; contract that justifies that stronger claim, the answer remains Canon 0.

((expr . "(let ((rules (quote (((parent alice bob)))))) (reason-observe (quote (parent bob alice)) rules))")
 (expected . "()")
 (active . t)
 (law . no-evidence-is-not-unknown))
