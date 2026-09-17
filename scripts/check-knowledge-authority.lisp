; #383 — Lisp-owned knowledge artifact authority classification checker.
; GREEN slice: newly observed artifacts require explicit classification rows.
; RED slice: every classification row must carry all required fields.

(def knowledge-authority-field-from
  (lambda (name fields)
    (cond
      ((atom fields) (structural-kind empty-list) (quote missing))
      ((atom fields) (structural-kind atom) (quote missing))
      ((atom fields) (structural-kind pair)
       (let ((field (car fields)))
         (cond
           ((eq (car field) name) (identity-relation same) (second field))
           ((eq (car field) name) (identity-relation distinct)
            (knowledge-authority-field-from name (cdr fields)))))))))

(def knowledge-authority-field
  (lambda (name row)
    (knowledge-authority-field-from name (cdr row))))

(def knowledge-authority-find-row-by-path
  (lambda (path rows)
    (cond
      ((atom rows) (structural-kind empty-list) (quote ()))
      ((atom rows) (structural-kind atom) (quote ()))
      ((atom rows) (structural-kind pair)
       (let ((row (car rows)))
         (cond
           ((equal? path (knowledge-authority-field (quote path) row))
            (structural-relation same)
            row)
           ((equal? path (knowledge-authority-field (quote path) row))
            (structural-relation distinct)
            (knowledge-authority-find-row-by-path path (cdr rows)))))))))

(def knowledge-authority-violation
  (lambda (kind detail)
    (list (quote knowledge-authority-violation) kind detail)))

(def knowledge-authority-observed-coverage-verdict
  (lambda (rows observed)
    (cond
      ((atom observed) (structural-kind empty-list)
       (list (quote knowledge-authority-ok)))
      ((atom observed) (structural-kind atom)
       (knowledge-authority-violation (quote malformed-observed-list) observed))
      ((atom observed) (structural-kind pair)
       (let* ((name (car observed))
              (path (string-append "knowledge/" name))
              (found (knowledge-authority-find-row-by-path path rows)))
         (cond
           ((atom found) (structural-kind empty-list)
            (knowledge-authority-violation (quote unclassified-artifact) path))
           ((atom found) (structural-kind atom)
            (knowledge-authority-violation (quote malformed-row) path))
           ((atom found) (structural-kind pair)
            (knowledge-authority-observed-coverage-verdict rows (cdr observed)))))))))

(def knowledge-authority-sample-row
  (quote
    (artifact
      (path "knowledge/a.lisp")
      (class operational-reference)
      (scope sample)
      (authority-source (issue 383))
      (lifecycle active)
      (consumers ()))))

(def knowledge-authority-sample-missing-class
  (quote
    (artifact
      (path "knowledge/a.lisp")
      (scope sample)
      (authority-source (issue 383))
      (lifecycle active)
      (consumers ()))))

(def knowledge-authority-selftest-unclassified
  (lambda ()
    (knowledge-authority-observed-coverage-verdict
      (list knowledge-authority-sample-row)
      (quote ("a.lisp" "b.lisp")))))

; Intentionally RED: coverage alone cannot detect a present row with missing schema fields.
(def knowledge-authority-selftest-missing-class
  (lambda ()
    (knowledge-authority-observed-coverage-verdict
      (list knowledge-authority-sample-missing-class)
      (quote ("a.lisp")))))

(def knowledge-authority-assert-verdict
  (lambda (actual expected)
    (cond
      ((equal? actual expected) (structural-relation same)
       (list (quote knowledge-authority-selftest-ok)))
      ((equal? actual expected) (structural-relation distinct)
       (let ((shown (print actual)))
         (car (quote ())))))))

(knowledge-authority-assert-verdict
  (knowledge-authority-selftest-unclassified)
  (quote
    (knowledge-authority-violation
      unclassified-artifact
      "knowledge/b.lisp")))

(knowledge-authority-assert-verdict
  (knowledge-authority-selftest-missing-class)
  (quote (knowledge-authority-violation missing-field class)))

(print (quote (knowledge-authority-selftests-ok unclassified-artifact missing-field)))
