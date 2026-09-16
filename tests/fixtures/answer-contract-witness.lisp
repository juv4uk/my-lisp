; #228 — Lisp-owned executable witness for the layered answer-contract data.
;
; contracts/answer-contract.lisp is pure data. The host test transports its
; UTF-8 source text into Lisp and defines `answer-contract-document` through
; the Lisp reader. This witness owns every semantic expectation below; Rust
; owns only file-byte transport because core my-lisp intentionally has no
; implicit filesystem capability.

(def answer-contract-schema
  (lambda () (car answer-contract-document)))

(def answer-contract-entries
  (lambda () (cdr answer-contract-document)))

(def answer-contract-field
  (lambda (entry field)
    (let ((found (assoc field entry)))
      (cond
        ((atom found) (quote ()))
        (t (cdr found))))))

(def answer-contract-find
  (lambda (identity entries)
    (cond
      ((atom entries) (quote ()))
      ((equal?
         (answer-contract-field (car entries) (quote identity))
         identity)
       (car entries))
      (t (answer-contract-find identity (cdr entries))))))

(def answer-contract-entry
  (lambda (identity)
    (answer-contract-find identity (answer-contract-entries))))

(def answer-contract-witness-record
  (lambda (status detail)
    (list
      (quote answer-contract-witness)
      (list (quote status) status)
      (list (quote detail) detail))))

(def answer-contract-witness-expect
  (lambda (identity field expected)
    (let ((entry (answer-contract-entry identity)))
      (cond
        ((atom entry)
         (list (quote missing-entry) identity field expected))
        ((equal? (answer-contract-field entry field) expected)
         (quote ()))
        (t
         (list
           (quote mismatch)
           identity
           field
           expected
           (answer-contract-field entry field)))))))

(def answer-contract-witness-expect-missing
  (lambda (identity)
    (cond
      ((atom (answer-contract-entry identity)) (quote ()))
      (t (list (quote unexpected-entry) identity)))))

(def answer-contract-witness-first-failure
  (lambda (checks)
    (cond
      ((atom checks) (quote ()))
      ((atom (car checks))
       (answer-contract-witness-first-failure (cdr checks)))
      (t (car checks)))))

(def answer-contract-witness
  (lambda ()
    (let ((failure
            (answer-contract-witness-first-failure
              (list
                (answer-contract-witness-expect
                  (quote canon-zero)
                  (quote domain-owner)
                  (quote no-answer-boundary))
                (answer-contract-witness-expect
                  (quote canon-zero)
                  (quote result-form)
                  (quote empty-list))
                (answer-contract-witness-expect
                  "0004"
                  (quote domain-owner)
                  (quote structure))
                (answer-contract-witness-expect
                  "0004"
                  (quote result-form)
                  (quote pair))
                (answer-contract-witness-expect
                  "0002"
                  (quote domain-owner)
                  (quote structural-observation))
                (answer-contract-witness-expect
                  "1014"
                  (quote domain-owner)
                  (quote exact-q-decision))
                (answer-contract-witness-expect
                  "1014"
                  (quote binary-values)
                  (quote ("0/1" "1/1")))
                (answer-contract-witness-expect
                  "1014"
                  (quote outside-domain)
                  (quote delegate))
                (answer-contract-witness-expect
                  "0104"
                  (quote domain-owner)
                  (quote mathematical-result))
                (answer-contract-witness-expect
                  "1118"
                  (quote domain-owner)
                  (quote non-mathematical-reasoning))
                (answer-contract-witness-expect
                  "1118"
                  (quote no-answer)
                  (quote ()))
                (answer-contract-witness-expect
                  "0007"
                  (quote domain-owner)
                  (quote control-consumer))
                (answer-contract-witness-expect
                  "0007"
                  (quote generic-value-coercion)
                  (quote forbidden))
                ; Human spellings stay in semantic-registry.lisp. The contract
                ; is keyed by semantic identity only. Canon 0 is the one special
                ; non-ID entry because the empty list has no lexical Canon ID.
                (answer-contract-witness-expect-missing (quote cons))
                (answer-contract-witness-expect-missing (quote reason))))))
      (cond
        ((eq (answer-contract-schema) (quote answer-contract/1))
         (cond
           ((atom failure)
            (answer-contract-witness-record (quote pass) (quote first-slice)))
           (t
            (answer-contract-witness-record (quote fail) failure))))
        (t
         (answer-contract-witness-record
           (quote fail)
           (list (quote schema) (answer-contract-schema))))))))
