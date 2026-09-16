; #218 — Lisp-owned verifier for contracts/structural-observation-contract.lisp.
; The host transports the contract document into `structural-observation-document`.

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

(def so-expect
  (lambda (identity field expected)
    (let ((entry (so-entry identity)))
      (cond
        ((atom entry) (list (quote missing-entry) identity))
        ((equal? (so-field entry field) expected) (quote ()))
        (t (list (quote mismatch) identity field expected
                 (so-field entry field)))))))

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
                (so-expect "0002" (quote generic-truth-coercion) (quote forbidden))
                (so-expect "0002" (quote control-dispatch) (quote delegated-to-217))
                (so-expect "0003" (quote input-domain) (quote (atom atom)))
                (so-expect "0003" (quote result-form) (quote identity-relation))
                (so-expect "0003" (quote outside-domain) (quote type-error))
                (so-expect "0003" (quote generic-truth-coercion) (quote forbidden))
                (so-expect "0003" (quote control-dispatch) (quote delegated-to-217))))))
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
