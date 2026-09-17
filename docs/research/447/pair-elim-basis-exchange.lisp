; #447 single pair-eliminator research model
; Research-only data. NOT semantic authority. NO Canon/runtime change.
;
; Question:
;   Does replacing independent CAR/CDR primitives with one product eliminator
;   remove semantic power, or only choose an equivalent basis?
;
; The candidate is intentionally abstract and has no proposed public spelling,
; semantic ID, ABI, implementation, or ratification status.

(pair-elim-research
  (schema-version 1)
  (issue 447)
  (role research-only)
  (production-change none)

  (existing-basis
    (pair-construction cons)
    (pair-elimination (car cdr))
    (function-capability (lambda application)))

  (candidate-basis
    (pair-construction cons)
    (pair-elimination pair-elim)
    (function-capability (lambda application)))

  (candidate-law
    (pair-elim (pair left right) selector)
    (result (selector left right))
    (non-pair named-type-failure)
    (selector-evaluated-only-after-pair-admission t))

  (candidate-to-current
    (car-derived
      (lambda (p)
        (pair-elim p (lambda (left right) left))))
    (cdr-derived
      (lambda (p)
        (pair-elim p (lambda (left right) right)))))

  (current-to-candidate
    (pair-elim-derived
      (lambda (p selector)
        (selector (car p) (cdr p)))))

  (falsification-corpus
    (ordinary-pair (a . b))
    (improper-pair (a . b))
    (proper-list-like (a b c))
    (nested-pair ((a . b) . (c . d)))
    (fresh-left (fresh-left . stable-right))
    (fresh-right (stable-left . fresh-right))
    (non-pair () atom-value string-value number-value))

  (required-observations
    (derived-car equals-current-car-on-admitted-pairs)
    (derived-cdr equals-current-cdr-on-admitted-pairs)
    (derived-car preserves-left-fresh-payload)
    (derived-cdr preserves-right-fresh-payload)
    (candidate-non-pair-failure matches-projector-domain-boundary)
    (reverse-derivation reproduces-candidate-on-pairs))

  (circularity-guards
    (candidate-definition may-not-use car cdr)
    (derived-car may-use pair-elim but-not car cdr peer-surfaces registry-roundtrip host-destructuring backend-load)
    (derived-cdr may-use pair-elim but-not car cdr peer-surfaces registry-roundtrip host-destructuring backend-load)
    (reverse-derivation explicitly-uses-current-projectors-for-equivalence-test-only))

  (power-accounting
    (before "two primitive projection entry points expose left/right coordinates")
    (after "one eliminator exposes both coordinates to an already-existing two-argument function application")
    (bidirectional-definability expected)
    (expressive-closure expected-equal)
    (primitive-entry-count 2-to-1)
    (semantic-power-removal expected-none))

  (classification-before-execution insufficient-evidence)
  (classification-rule
    (if bidirectional-equivalence-and-no-new-observable-law
        basis-exchange-only)
    (if candidate-to-current-only-and-strictly-weaker-observable-laws
        strict-compression)
    (if candidate-needs-extra-hidden-control-or-representation-power
        stronger-hidden-primitive)
    (otherwise insufficient-evidence))

  (claim-boundary
    "A one-name primitive set is not automatically a smaller semantic basis. Bidirectional definability with unchanged expressive closure is evidence for basis exchange, even when primitive entry count decreases."))
