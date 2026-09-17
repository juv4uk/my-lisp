; #220 — epistemic retrieval and round-trip semantics belong to Lisp.
; Rust may observe that this witness passes, but it must not author structural
; same/distinct verdicts or use historical t/() as the semantic oracle.

(def epistemic-authority-rows
  (lambda ()
    (list
      (list
        (quote supporting-evidence-simple-match)
        (equal?
          (supporting-evidence
            (make-evidence
              (quote (claim-ref cml-build-available))
              (quote live-test)
              (quote supports)
              (quote (digest "d")))
            (quote (claim-ref cml-build-available)))
          (make-evidence
            (quote (claim-ref cml-build-available))
            (quote live-test)
            (quote supports)
            (quote (digest "d"))))
        (quote (structural-relation same)))
      (list
        (quote supporting-evidence-non-support-is-empty)
        (atom
          (supporting-evidence
            (make-evidence
              (quote (claim-ref cml-build-available))
              (quote live-test)
              (quote contradicts)
              (quote (digest "d")))
            (quote (claim-ref cml-build-available))))
        (quote (structural-kind empty-list)))
      (list
        (quote supporting-evidence-claim-mismatch-is-empty)
        (atom
          (supporting-evidence
            (make-evidence
              (quote (claim-ref cml-build-available))
              (quote live-test)
              (quote supports)
              (quote (digest "d")))
            (quote (claim-ref some-other-claim))))
        (quote (structural-kind empty-list)))
      (list
        (quote supporting-evidence-structural-claim-ref)
        (equal?
          (supporting-evidence
            (make-evidence
              (quote
                (claim-ref
                  (claim
                    (statement (build cml succeeds))
                    (source (observation local-run))
                    (review proposed))))
              (quote live-test)
              (quote supports)
              (quote (digest "d")))
            (quote
              (claim-ref
                (claim
                  (statement (build cml succeeds))
                  (source (observation local-run))
                  (review proposed)))))
          (make-evidence
            (quote
              (claim-ref
                (claim
                  (statement (build cml succeeds))
                  (source (observation local-run))
                  (review proposed))))
            (quote live-test)
            (quote supports)
            (quote (digest "d"))))
        (quote (structural-relation same)))
      (list
        (quote observation-write-read-roundtrip)
        (equal?
          (read
            (write-to-string
              (make-observation
                (quote (digest "sha256:abc"))
                (quote (build cml succeeds)))))
          (make-observation
            (quote (digest "sha256:abc"))
            (quote (build cml succeeds))))
        (quote (structural-relation same)))
      (list
        (quote claim-write-read-roundtrip)
        (equal?
          (read
            (write-to-string
              (make-claim
                (quote (build cml succeeds))
                (quote (observation local-run))
                (quote proposed))))
          (make-claim
            (quote (build cml succeeds))
            (quote (observation local-run))
            (quote proposed)))
        (quote (structural-relation same)))
      (list
        (quote evidence-write-read-roundtrip)
        (equal?
          (read
            (write-to-string
              (make-evidence
                (quote (claim-ref cml-build-available))
                (quote live-test)
                (quote supports)
                (quote (test (fixture conformance.my) (case exact-rational-division))))))
          (make-evidence
            (quote (claim-ref cml-build-available))
            (quote live-test)
            (quote supports)
            (quote (test (fixture conformance.my) (case exact-rational-division)))))
        (quote (structural-relation same)))
      (list
        (quote intent-write-read-roundtrip)
        (equal?
          (read
            (write-to-string
              (make-intent
                (quote (build cml))
                (quote (process:cargo tcp-client))
                (quote (missing-capability))
                (quote (build-artifact cml)))))
          (make-intent
            (quote (build cml))
            (quote (process:cargo tcp-client))
            (quote (missing-capability))
            (quote (build-artifact cml))))
        (quote (structural-relation same)))
      (list
        (quote canon-zero-write-read-roundtrip)
        (eq
          (read (write-to-string (quote ())))
          (quote ()))
        (quote (identity-relation same)))
      (list
        (quote identity-result-write-read-roundtrip)
        (equal?
          (read (write-to-string (eq 1 1)))
          (eq 1 1))
        (quote (structural-relation same)))
      (list
        (quote exact-q-no-write-read-roundtrip)
        (eq
          (read (write-to-string (< 2 1)))
          (< 2 1))
        (quote (identity-relation same)))
      (list
        (quote exact-q-yes-write-read-roundtrip)
        (eq
          (read (write-to-string (< 1 2)))
          (< 1 2))
        (quote (identity-relation same))))))

(def epistemic-authority-check
  (lambda (rows)
    (cond
      ((atom rows) (structural-kind empty-list)
       (quote (epistemic-authority-witness (status pass))))
      ((atom rows) (structural-kind pair)
       (let ((row (car rows)))
         (cond
           ((equal? (second row) (third row)) (structural-relation same)
            (epistemic-authority-check (cdr rows)))
           ((equal? (second row) (third row)) (structural-relation distinct)
            (list
              (quote epistemic-authority-witness)
              (list (quote status) (quote fail))
              (list (quote case) (car row))
              (list (quote actual) (second row))
              (list (quote expected) (third row))))))))))

(def epistemic-authority-witness
  (lambda ()
    (epistemic-authority-check (epistemic-authority-rows))))
