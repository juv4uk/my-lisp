; #382 — Lisp-owned repository tooling inventory validator.
; GREEN slices: unregistered-tool + duplicate-path.
; RED-first: stale-path is intentionally not implemented yet.

(def repo-tooling-field-from
  (lambda (name fields)
    (cond
      ((atom fields) (structural-kind empty-list) (quote missing))
      ((atom fields) (structural-kind atom) (quote missing))
      ((atom fields) (structural-kind pair)
       (let ((field (car fields)))
         (cond
           ((eq (car field) name) (identity-relation same) (second field))
           ((eq (car field) name) (identity-relation distinct)
            (repo-tooling-field-from name (cdr fields)))))))))

(def repo-tooling-field
  (lambda (name row)
    (repo-tooling-field-from name (cdr row))))

(def repo-tooling-find-row-by-path
  (lambda (path rows)
    (cond
      ((atom rows) (structural-kind empty-list) (quote ()))
      ((atom rows) (structural-kind atom) (quote ()))
      ((atom rows) (structural-kind pair)
       (let ((row (car rows)))
         (cond
           ((equal? path (repo-tooling-field (quote path) row))
            (structural-relation same)
            row)
           ((equal? path (repo-tooling-field (quote path) row))
            (structural-relation distinct)
            (repo-tooling-find-row-by-path path (cdr rows)))))))))

(def repo-tooling-violation
  (lambda (kind detail)
    (list (quote repo-tooling-violation) kind detail)))

(def repo-tooling-observed-coverage-verdict
  (lambda (rows observed)
    (cond
      ((atom observed) (structural-kind empty-list) (list (quote repo-tooling-ok)))
      ((atom observed) (structural-kind atom)
       (repo-tooling-violation (quote malformed-observed-list) observed))
      ((atom observed) (structural-kind pair)
       (let* ((name (car observed))
              (path (string-append "scripts/" name))
              (found (repo-tooling-find-row-by-path path rows)))
         (cond
           ((atom found) (structural-kind empty-list)
            (repo-tooling-violation (quote unregistered-tool) path))
           ((atom found) (structural-kind atom)
            (repo-tooling-violation (quote malformed-row) path))
           ((atom found) (structural-kind pair)
            (repo-tooling-observed-coverage-verdict rows (cdr observed)))))))))

(def repo-tooling-duplicate-path-verdict
  (lambda (rows)
    (cond
      ((atom rows) (structural-kind empty-list) (list (quote repo-tooling-ok)))
      ((atom rows) (structural-kind atom)
       (repo-tooling-violation (quote malformed-inventory-list) rows))
      ((atom rows) (structural-kind pair)
       (let* ((row (car rows))
              (path (repo-tooling-field (quote path) row))
              (found (repo-tooling-find-row-by-path path (cdr rows))))
         (cond
           ((atom found) (structural-kind empty-list)
            (repo-tooling-duplicate-path-verdict (cdr rows)))
           ((atom found) (structural-kind atom)
            (repo-tooling-violation (quote malformed-row) path))
           ((atom found) (structural-kind pair)
            (repo-tooling-violation (quote duplicate-path) path))))))))

(def repo-tooling-verdict
  (lambda (rows observed)
    (let ((duplicate-verdict (repo-tooling-duplicate-path-verdict rows)))
      (cond
        ((equal? duplicate-verdict (list (quote repo-tooling-ok)))
         (structural-relation same)
         (repo-tooling-observed-coverage-verdict rows observed))
        ((equal? duplicate-verdict (list (quote repo-tooling-ok)))
         (structural-relation distinct)
         duplicate-verdict)))))

(def repo-tooling-sample-row-a
  (quote
    (tool
      (path "scripts/a.lisp")
      (kind check)
      (language lisp)
      (role sample)
      (lifecycle active)
      (callers ())
      (authority-source (issue 382))
      (migration-issue ())
      (replacement ())
      (removal-condition ()))))

(def repo-tooling-sample-row-c
  (quote
    (tool
      (path "scripts/c.lisp")
      (kind helper)
      (language lisp)
      (role stale-sample)
      (lifecycle active)
      (callers ())
      (authority-source (issue 382))
      (migration-issue ())
      (replacement ())
      (removal-condition ()))))

(def repo-tooling-selftest-unregistered
  (lambda ()
    (repo-tooling-verdict
      (list repo-tooling-sample-row-a)
      (quote ("a.lisp" "b.lisp")))))

(def repo-tooling-selftest-duplicate-path
  (lambda ()
    (repo-tooling-verdict
      (list repo-tooling-sample-row-a repo-tooling-sample-row-a)
      (quote ("a.lisp")))))

(def repo-tooling-selftest-stale-path
  (lambda ()
    (repo-tooling-verdict
      (list repo-tooling-sample-row-a repo-tooling-sample-row-c)
      (quote ("a.lisp")))))

(def repo-tooling-assert-verdict
  (lambda (actual expected)
    (cond
      ((equal? actual expected) (structural-relation same)
       (list (quote repo-tooling-selftest-ok)))
      ((equal? actual expected) (structural-relation distinct)
       (let ((shown (print actual)))
         (car (quote ())))))))

(repo-tooling-assert-verdict
  (repo-tooling-selftest-unregistered)
  (quote (repo-tooling-violation unregistered-tool "scripts/b.lisp")))

(repo-tooling-assert-verdict
  (repo-tooling-selftest-duplicate-path)
  (quote (repo-tooling-violation duplicate-path "scripts/a.lisp")))

; RED: a registered path absent from observed scripts must not be accepted.
(repo-tooling-assert-verdict
  (repo-tooling-selftest-stale-path)
  (quote (repo-tooling-violation stale-path "scripts/c.lisp")))

(print (quote (repo-tooling-selftests-ok unregistered-tool duplicate-path stale-path)))
