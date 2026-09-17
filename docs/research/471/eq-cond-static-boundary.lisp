; #471 EQ vs canonical three-part COND
; Research-only. NOT semantic authority. NO production change.
;
; Question: can COND's explicit-result equality replace EQ's dynamic
; two-runtime-operand atomic identity relation?
;
; Pre-execution record. Predictions are written before the live run.

(eq-cond-static-boundary/1
  (issue 471)
  (parent 419)
  (related 218)
  (role research-only)

  (eq-capability
    (operands left-runtime right-runtime)
    (input-domain (atom atom))
    (result-domain identity-relation)
    (results same distinct)
    (pair-outside-domain Type))

  (cond-capability
    (canonical-clause-shape (query expected-source-datum expression))
    (actual query-runtime-evaluated)
    (expected quoted-source-datum)
    (selection-rule actual==quoted-expected)
    (generic-truth-coercion forbidden))

  (hypothesis-under-test
    "Canonical COND may look like a hidden equality oracle, but its right-hand side is syntax/data, not a second runtime operand.")

  (predictions
    (static-atom
      (source "(cond ((quote radio) radio (quote selected)) ((quote fallback) fallback (quote missed)))")
      (expected selected))
    (static-pair
      (source "(cond ((quote (radio . antenna)) (radio . antenna) (quote selected)) ((quote fallback) fallback (quote missed)))")
      (expected selected))
    (eq-pair
      (source "(eq (quote (radio . antenna)) (quote (radio . antenna)))")
      (expected-error Type))
    (dynamic-same-false-negative
      (source "((lambda (left right) (cond (left right (quote same)) ((quote fallback) fallback (quote distinct)))) (quote radio) (quote radio))")
      (runtime-left radio)
      (runtime-right radio)
      (quoted-expected-datum right)
      (expected-result distinct))
    (dynamic-distinct-false-positive
      (source "((lambda (left right) (cond (left right (quote same)) ((quote fallback) fallback (quote distinct)))) (quote right) (quote radio))")
      (runtime-left right)
      (runtime-right radio)
      (quoted-expected-datum right)
      (expected-result same)))

  (classification-before-run insufficient-evidence)

  (non-claims
    no-global-eq-irreducibility-proof
    no-cond-reclassification
    no-canon-change-authorized
    no-production-change-authorized))
