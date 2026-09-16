; #219 — Lisp-owned unification outcome contract.
; `unify` remains the historical mechanism. `unify-observe` makes the
; algorithm's established result explicit without confusing empty substitution
; with Canon-0 no-answer and without inventing a cause for failure.

(unification-outcome-contract/1
  ((query . unify-observe)
   (input . (left right substitution))
   (result-forms . ((unified substitution)
                    (unification-failure left right substitution)))
   (bare-canon-zero-result . forbidden-for-completed-query)
   (generic-truth-coercion . forbidden))

  ((law . empty-substitution-is-success-data)
   (raw-result . ())
   (observed-result . (unified ()))
   (not-equal . ())))

  ((law . established-unification-failure-is-not-canon-zero)
   (raw-result . fail)
   (observed-result-form . (unification-failure left right substitution))
   (cause . not-invented)
   (not-equal . ()))
)
