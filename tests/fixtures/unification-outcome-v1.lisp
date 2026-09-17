; #219 — explicit unification outcomes.
; A completed unification query must preserve the distinction between an empty
; successful substitution and a positively established algorithmic failure.

((expr . "(unify-observe (quote radio) (quote radio) (quote ()))")
 (expected . "(unified ())")
 (active . t)
 (law . empty-substitution-is-success-data))

((expr . "(unify-observe (logic-var (quote x)) (quote alice) (quote ()))")
 (expected . "(unified ((x . alice)))")
 (active . t)
 (law . substitution-success-is-explicit))

((expr . "(unify-observe (quote radio) (quote antenna) (quote ()))")
 (expected . "(unification-failure radio antenna ())")
 (active . t)
 (law . established-failure-is-explicit))

((expr . "(unify-observe (logic-var (quote x)) (quote (f (var x))) (quote ()))")
 (expected . "(unification-failure (var x) (f (var x)) ())")
 (active . t)
 (law . occurs-check-failure-is-explicit))
