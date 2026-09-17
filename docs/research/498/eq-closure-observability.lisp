; #498 bounded closure-identity observability without EQ
; Research-only. NOT semantic authority. NO production change.
; Predictions are authored before the live run.

(eq-closure-observability/1
  (issue 498)
  (parent 471)
  (grandparent 419)
  (role research-only)

  (witness-pair
    (left (lambda (x) x))
    (right (lambda (x) x))
    (allocation separately-created))

  (target-predictions
    (reused-closure (identity-relation same))
    (fresh-left-vs-fresh-right (identity-relation distinct)))

  (lower-basis
    (allowed
      atom
      application
      quote
      eval
      write-to-string
      cons
      car
      cdr
      cond
      lambda/application)
    (forbidden
      eq
      eq-peer-surfaces
      semantic-registry-roundtrip
      host-Value-equality
      Rc::ptr_eq
      pointer-or-address-observation
      debug-identity))

  (authored-context-corpus
    (atom-classification
      (predicted-left (structural-kind atom))
      (predicted-right (structural-kind atom)))
    (application-radio
      (predicted-left radio)
      (predicted-right radio))
    (application-empty
      (predicted-left ())
      (predicted-right ()))
    (application-number
      (predicted-left 42)
      (predicted-right 42))
    (application-dotted-pair
      (predicted-left (radio . antenna))
      (predicted-right (radio . antenna)))
    (eval-pass-through-application
      (predicted-left radio)
      (predicted-right radio))
    (serialization
      (predicted-left "<lambda>")
      (predicted-right "<lambda>"))
    (cons-car-serialization
      (predicted-left "<lambda>")
      (predicted-right "<lambda>"))
    (cons-cdr
      (predicted-left tail-marker)
      (predicted-right tail-marker))
    (static-cond-lambda-datum
      (predicted-left no-static-match)
      (predicted-right no-static-match))
    (car-domain-error
      (predicted-left Type)
      (predicted-right Type)))

  (falsification-rule
    "Any lower context in the authored corpus that distinguishes left from right defeats this bounded indistinguishability witness. If all coincide while EQ differs, record bounded-indistinguishability-witness only.")

  (classification-before-run insufficient-evidence)

  (non-claims
    no-global-contextual-equivalence
    no-global-eq-irreducibility
    no-exhaustive-context-search
    no-production-change-authorized))
