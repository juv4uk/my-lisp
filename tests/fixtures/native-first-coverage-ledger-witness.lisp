; #508 — executable authority witness for the native coverage ledger.
; The ledger may describe routing, but it may not create it:
;   native-supported -> classifier native-plan + CPU native route + #509 parity
;   fallback/blocked -> classifier evaluator-fallback of the unchanged source
; Blocked rows must also name a concrete prerequisite.

(load "lib/core.lisp")
(load "lib/machine/encoding/x86-64.lisp")
(load "lib/machine/layout/pair-x86-64.lisp")
(load "lib/machine/operands/x86-64.lisp")
(load "lib/machine/admission/x86-64.lisp")
(load "lib/machine/lowering/semantic-x86-64.lisp")
(load "lib/machine/dispatch/native-first.lisp")
(load "lib/machine/dispatch/native-first-execute.lisp")
(load "lib/machine/dispatch/native-first-parity.lisp")
(load "lib/machine/dispatch/native-first-coverage.lisp")

(def native-first-coverage-same?
  (lambda (left right)
    (cond
      ((equal? left right) (structural-relation same) t)
      ((equal? left right) (structural-relation distinct) (quote ())))))

(def native-first-coverage-field
  (lambda (name row)
    (let ((found (assoc name (cdr row))))
      (cond
        ((atom found) (structural-kind empty-list) (quote ()))
        ((atom found) (structural-kind atom) (quote ()))
        ((atom found) (structural-kind pair) (second found))))))

(def native-first-coverage-present?
  (lambda (value)
    (cond
      ((native-first-coverage-same? value (quote ())) (quote ()))
      (t t))))

(def native-first-coverage-fallback-plan-valid?
  (lambda (expression)
    (native-first-coverage-same?
      (native-first-plan expression)
      (list (quote evaluator-fallback) expression))))

(def native-first-coverage-native-row-valid?
  (lambda (row)
    (let* ((class (native-first-coverage-field (quote class) row))
           (expression
             (native-first-coverage-field
               (quote representative)
               row))
           (expected
             (native-first-coverage-field
               (quote expected)
               row))
           (effect
             (native-first-coverage-field
               (quote effect)
               row))
           (error-class
             (native-first-coverage-field
               (quote error)
               row))
           (evidence
             (native-first-coverage-field
               (quote evidence)
               row))
           (plan (native-first-plan expression))
           (outcome
             (native-first-execute-expression expression))
           (parity
             (native-first-parity-case
               (list
                 class
                 expression
                 expected
                 effect
                 error-class))))
      (and
        (native-first-coverage-present? class)
        (native-first-coverage-present? expected)
        (native-first-coverage-same?
          evidence
          (quote native-first-parity-witness))
        (cond
          ((atom plan) (structural-kind empty-list) (quote ()))
          ((atom plan) (structural-kind atom) (quote ()))
          ((atom plan) (structural-kind pair)
           (cond
             ((eq (car plan) (quote native-plan))
              (identity-relation same)
              t)
             ((eq (car plan) (quote native-plan))
              (identity-relation distinct)
              (quote ())))))
        (native-first-coverage-same?
          outcome
          (list
            (quote execution-route)
            (quote native)
            (quote (status completed))
            (list (quote value) expected)))
        (cond
          ((atom parity) (structural-kind empty-list) (quote ()))
          ((atom parity) (structural-kind atom) (quote ()))
          ((atom parity) (structural-kind pair)
           (cond
             ((native-first-coverage-same?
                (third parity)
                (quote pass))
              t)
             (t (quote ())))))))))

(def native-first-coverage-fallback-row-valid?
  (lambda (row)
    (let ((expression
            (native-first-coverage-field
              (quote representative)
              row))
          (reason
            (native-first-coverage-field
              (quote reason)
              row))
          (evidence
            (native-first-coverage-field
              (quote evidence)
              row)))
      (and
        (native-first-coverage-present? reason)
        (native-first-coverage-same?
          evidence
          (quote native-first-dispatch-witness))
        (native-first-coverage-fallback-plan-valid?
          expression)))))

(def native-first-coverage-blocked-row-valid?
  (lambda (row)
    (and
      (native-first-coverage-fallback-row-valid? row)
      (native-first-coverage-present?
        (native-first-coverage-field
          (quote prerequisite)
          row)))))

(def native-first-coverage-row-valid?
  (lambda (row)
    (cond
      ((atom row) (structural-kind empty-list) (quote ()))
      ((atom row) (structural-kind atom) (quote ()))
      ((atom row) (structural-kind pair)
       (cond
         ((eq (car row) (quote native-coverage))
          (identity-relation same)
       (let ((status
               (native-first-coverage-field
                 (quote status)
                 row)))
         (cond
           ((eq status (quote native-supported))
            (identity-relation same)
            (native-first-coverage-native-row-valid? row))
           ((eq status (quote fallback-required))
            (identity-relation same)
            (native-first-coverage-fallback-row-valid? row))
           ((eq status (quote blocked-runtime-prerequisite))
            (identity-relation same)
            (native-first-coverage-blocked-row-valid? row))
           (t (quote ())))))
         ((eq (car row) (quote native-coverage))
          (identity-relation distinct)
          (quote ())))))))

(def native-first-coverage-all-valid?
  (lambda (rows)
    (cond
      ((atom rows) (structural-kind empty-list) t)
      ((atom rows) (structural-kind atom) (quote ()))
      ((atom rows) (structural-kind pair)
       (cond
         ((native-first-coverage-row-valid? (car rows))
          (native-first-coverage-all-valid? (cdr rows)))
         (t (quote ())))))))

(def native-first-coverage-count-status-onto
  (lambda (rows status count)
    (cond
      ((atom rows) (structural-kind empty-list) count)
      ((atom rows) (structural-kind atom) count)
      ((atom rows) (structural-kind pair)
       (cond
         ((eq
            (native-first-coverage-field
              (quote status)
              (car rows))
            status)
          (identity-relation same)
          (native-first-coverage-count-status-onto
            (cdr rows)
            status
            (+ count 1)))
         ((eq
            (native-first-coverage-field
              (quote status)
              (car rows))
            status)
          (identity-relation distinct)
          (native-first-coverage-count-status-onto
            (cdr rows)
            status
            count)))))))

(def native-first-coverage-count-status
  (lambda (status)
    (native-first-coverage-count-status-onto
      native-first-coverage-ledger
      status
      0)))

(def native-first-coverage-dispatch-independent?
  (and
    (not
      (string-contains?
        "native-first-coverage"
        (read-file "lib/machine/dispatch/native-first.lisp")))
    (not
      (string-contains?
        "native-first-coverage"
        (read-file "lib/machine/dispatch/native-first-execute.lisp")))
    (not
      (string-contains?
        "native-first-coverage"
        (read-file "lib/machine/dispatch/native-first-parity.lisp")))))

(def native-first-coverage-ledger-witness
  (lambda ()
    (let ((native-count
            (native-first-coverage-count-status
              (quote native-supported)))
          (fallback-count
            (native-first-coverage-count-status
              (quote fallback-required)))
          (blocked-count
            (native-first-coverage-count-status
              (quote blocked-runtime-prerequisite))))
      (cond
        ((and
           native-first-coverage-dispatch-independent?
           (native-first-coverage-all-valid?
             native-first-coverage-ledger)
           (native-first-coverage-same?
             (length native-first-coverage-ledger)
             6)
           (native-first-coverage-same?
             native-count
             1)
           (native-first-coverage-same?
             fallback-count
             2)
           (native-first-coverage-same?
             blocked-count
             3))
         (list
           (quote native-first-coverage-ledger-witness)
           (quote (status pass))
           (list (quote rows) 6)
           (list (quote native) native-count)
           (list (quote fallback) fallback-count)
           (list (quote blocked) blocked-count)))
        (t
         (list
           (quote native-first-coverage-ledger-witness)
           (quote (status fail))
           (list (quote rows)
                 (length native-first-coverage-ledger))
           (list (quote native) native-count)
           (list (quote fallback) fallback-count)
           (list (quote blocked) blocked-count)))))))

(native-first-coverage-ledger-witness)
