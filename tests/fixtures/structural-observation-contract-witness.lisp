; #218 — Lisp-owned verifier for contracts/structural-observation-contract.lisp.
; The host transports the contract document into `structural-observation-document`.
;
; Case tables are verified one case at a time. This avoids asking the historical
; macro expander to reconstruct a nested dotted-alist literal merely to compare
; two pieces of semantic data. The contract remains authoritative; this witness
; only reads its fields and checks the required domain facts.

(def so-field
  (lambda (entry field)
    (let ((found (assoc field entry)))
      (cond
        ((atom found) (quote ()))
        (t (cdr found))))))

(def so-find
  (lambda (identity entries)
    (cond
      ((atom entries) (quote ()))
      ((equal? (so-field (car entries) (quote identity)) identity)
       (car entries))
      (t (so-find identity (cdr entries))))))

(def so-entry
  (lambda (identity)
    (so-find identity (cdr structural-observation-document))))

(def so-case-find
  (lambda (case-name cases)
    (cond
      ((atom cases) (quote ()))
      ((equal? (so-field (car cases) (quote when)) case-name)
       (car cases))
      (t (so-case-find case-name (cdr cases))))))

(def so-expect
  (lambda (identity field expected)
    (let ((entry (so-entry identity)))
      (cond
        ((atom entry) (list (quote missing-entry) identity))
        ((equal? (so-field entry field) expected) (quote ()))
        (t (list (quote mismatch) identity field expected
                 (so-field entry field)))))))

(def so-expect-case
  (lambda (identity case-name expected-result)
    (let ((entry (so-entry identity)))
      (cond
        ((atom entry)
         (list (quote missing-entry) identity))
        (t
         (let ((case-entry
                 (so-case-find case-name (so-field entry (quote cases)))))
           (cond
             ((atom case-entry)
              (list (quote missing-case) identity case-name))
             ((equal? (so-field case-entry (quote result)) expected-result)
              (quote ()))
             (t
              (list
                (quote case-mismatch)
                identity
                case-name
                expected-result
                (so-field case-entry (quote result)))))))))))

(def so-first-failure
  (lambda (checks)
    (cond
      ((atom checks) (quote ()))
      ((atom (car checks)) (so-first-failure (cdr checks)))
      (t (car checks)))))

(def structural-observation-contract-witness
  (lambda ()
    (let ((failure
            (so-first-failure
              (list
                (so-expect "0002" (quote result-form) (quote structural-kind))
                (so-expect-case
                  "0002"
                  (quote canon-zero)
                  (quote (structural-kind empty-list)))
                (so-expect-case
                  "0002"
                  (quote pair)
                  (quote (structural-kind pair)))
                (so-expect-case
                  "0002"
                  (quote non-pair-nonempty)
                  (quote (structural-kind atom)))
                (so-expect "0002" (quote generic-truth-coercion) (quote forbidden))
                (so-expect "0002" (quote control-dispatch) (quote explicit-result-equality))
                (so-expect "0003" (quote input-domain) (quote (atom atom)))
                (so-expect "0003" (quote result-form) (quote identity-relation))
                (so-expect-case
                  "0003"
                  (quote same-atom)
                  (quote (identity-relation same)))
                (so-expect-case
                  "0003"
                  (quote distinct-atoms)
                  (quote (identity-relation distinct)))
                (so-expect "0003" (quote outside-domain) (quote type-error))
                (so-expect "0003" (quote generic-truth-coercion) (quote forbidden))
                (so-expect "0003" (quote control-dispatch) (quote explicit-result-equality))))))
      (cond
        ((not (eq (car structural-observation-document)
                  (quote structural-observation-contract/1)))
         (list (quote structural-observation-contract-witness)
               (list (quote status) (quote fail))
               (list (quote detail) (quote schema))))
        ((atom failure)
         (list (quote structural-observation-contract-witness)
               (list (quote status) (quote pass))
               (list (quote detail) (quote atom-eq-domain-results))))
        (t
         (list (quote structural-observation-contract-witness)
               (list (quote status) (quote fail))
               (list (quote detail) failure)))))))
