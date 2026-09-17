; #452 pair-elim effect/evaluation-order boundary
; Research-only. NOT semantic authority. NO production change.
;
; Parent result #451 is intentionally preserved:
; pure extensional selector model => basis-exchange-only.
;
; This slice asks whether effectful selector *expressions* distinguish an
; ordinary callable eliminator from a stronger delayed-selector special form.

(pair-elim-effect-boundary
  (schema-version 2)
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

  (fresh-run
    (verification-pr 453)
    (actions-run 35282403011)
    (verification-head "d1f3b0ade20fb8d6deadcc0202425f5a79296af8")
    (merge-test-sha "ac46c0744d4b7ed744f526891d8fd1bca2dd847b")
    (runner ubuntu-24.04)
    (observer temporary-integration-test-removed-before-diff-check)
    (observer-tests 3)
    (observer-passed 3)
    (observer-failed 0)
    (ordinary-callable selector-expression-effect-before-body-domain-failure)
    (reverse-current selector-expression-effect-before-car-failure)
    (valid-reverse selector-body-effect-exactly-once)
    (diff-check pass))

  (observer-correction
    (first-run 35282247688)
    (status invalid-observer)
    (reason "CLI error path did not expose the Session output buffer, so absence of printed marker in process output was not semantic evidence")
    (correction "temporary Rust integration observer read Session.environment.output_snapshot() after eval_program returned Err; no production Rust was committed"))

  (classification
    (ordinary-callable pure-basis-exchange-extends-to-selector-expression-effects)
    (delayed-selector-form callback-candidate-observably-stronger))

  (classification-reason
    "The live evaluator executes the selector expression before an ordinary closure body and before CAR/CDR arguments on the reverse-current route. Both paths therefore preserve this observed effect order. A candidate that instead suppresses selector-expression evaluation until after pair admission differs observably and requires additional non-eager evaluation-control power."))

  (non-claims
    all-effect-kinds-not-exhausted
    allocation-resource-equivalence-not-measured
    asynchronous-or-host-effects-not-measured
    candidate-abi-not-proposed
    canon-change-not-authorized)

  (next-falsification
    "Keep any future pair-elim candidate ordinary/eager unless it explicitly declares and independently justifies extra control power; then separately test resource/allocation and failure-propagation observations."))

  (claim-boundary
    "Suppressing evaluation of the selector expression on non-pair input is not a free implementation detail: under the current eager application contract it is extra evaluation-control power."))
