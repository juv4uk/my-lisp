; #452 pair-elim effect/evaluation-order boundary
; Research-only. NOT semantic authority. NO production change.
;
; Parent result #451 is intentionally preserved:
; pure extensional selector model => basis-exchange-only.
;
; This slice asks whether effectful selector *expressions* distinguish an
; ordinary callable eliminator from a stronger delayed-selector special form.

(pair-elim-effect-boundary
  (schema-version 1)
  (issue 452)
  (role research-only)

  (current-evaluator-order
    (operator-before-ordinary-arguments t)
    (ordinary-arguments left-to-right)
    (first-failure-stops-later-arguments t)
    (source docs/adr/ADR-010-EVALUATOR-OUTCOME-PROVENANCE.md))

  (two-candidate-interpretations
    (ordinary-callable
      (surface-shape (pair-elim pair selector-expression))
      (evaluation "pair argument and selector-expression are evaluated before pair-elim body runs")
      (consequence "selector-expression effects can occur even when pair value is not admissible"))
    (delayed-selector-form
      (surface-shape (pair-elim pair selector-source))
      (evaluation "pair is admitted before selector source is evaluated")
      (consequence "requires non-eager evaluation-control power beyond ordinary function application")))

  (reverse-current-route
    (shape "selector-expression is operator of (selector (car p) (cdr p))")
    (consequence "operator expression is evaluated before CAR/CDR arguments; its effects occur before a later projector failure"))

  (falsification
    (nonpair p)
    (selector-expression-effect print-marker)
    (ordinary-callable-expected marker-before-domain-failure)
    (reverse-current-expected marker-before-car-failure)
    (delayed-selector-form-expected no-marker-before-domain-failure))

  (classification-rule
    (if ordinary-callable-matches-reverse-current
        pure-basis-exchange-extends-to-selector-expression-effects)
    (if delayed-selector-form-suppresses-selector-expression-effect
        callback-candidate-observably-stronger)
    (otherwise insufficient-evidence))

  (claim-boundary
    "Suppressing evaluation of the selector expression on non-pair input is not a free implementation detail: under the current eager application contract it is extra evaluation-control power."))
