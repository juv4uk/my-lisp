; #493 EQ serialization collision
; Research-only. NOT semantic authority. NO production change.
;
; Pre-execution record.

(eq-serialization-collision/1
  (issue 493)
  (parent 471)
  (grandparent 419)
  (role research-only)

  (target
    (identity "0003")
    (surface eq)
    (input-domain (atom atom))
    (result-domain identity-relation))

  (known-live-mechanism-to-test
    (closure-equality allocation-identity)
    (write-to-string-rendering closure "<lambda>"))

  (candidate-route
    (name serialized-text-equality)
    (allowed write-to-string string<? cond lambda/application quote)
    (forbidden eq eq-aliases host-Value-equality)
    (shape
      ((lambda (left right)
         ((lambda (left-text right-text)
            (cond
              ((string<? left-text right-text) ()
               (cond
                 ((string<? right-text left-text) () (quote text-same))
                 ((quote fallback) fallback (quote text-distinct))))
              ((quote fallback) fallback (quote text-distinct))))
          (write-to-string left)
          (write-to-string right)))
       LEFT RIGHT)))

  (predictions
    (same-closure
      (eq-result (identity-relation same)))
    (two-fresh-closures
      (eq-result (identity-relation distinct))
      (left-serialization "<lambda>")
      (right-serialization "<lambda>")
      (text-route text-same))
    (distinct-symbol-control
      (left radio)
      (right antenna)
      (text-route text-distinct)))

  (falsification-criterion
    "If two EQ-distinct admitted atoms serialize to the same text and the no-EQ text route classifies them text-same, serialization/text cannot preserve EQ semantics.")

  (classification-before-run insufficient-evidence)

  (non-claims
    no-global-eq-irreducibility-proof
    no-writer-bug-claim
    no-canon-change-authorized
    no-production-change-authorized))
