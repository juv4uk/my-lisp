; #382 — Lisp-owned repository tooling inventory validator.
; GREEN slices: unregistered-tool + duplicate-path + stale-path + closed enums
; + Python migration ownership.

(def repo-tooling-kinds
  (quote (check generator migration benchmark deploy release helper other)))

(def repo-tooling-languages
  (quote (lisp python shell javascript powershell other)))

(def repo-tooling-lifecycles
  (quote (active transitional legacy generated-helper archive-candidate)))

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

(def repo-tooling-symbol-admission
  (lambda (value admitted)
    (cond
      ((atom admitted) (structural-kind empty-list) (quote rejected))
      ((atom admitted) (structural-kind atom) (quote malformed-admitted-set))
      ((atom admitted) (structural-kind pair)
       (cond
         ((eq value (car admitted)) (identity-relation same) (quote admitted))
         ((eq value (car admitted)) (identity-relation distinct)
          (repo-tooling-symbol-admission value (cdr admitted))))))))

(def repo-tooling-row-enum-verdict
  (lambda (row)
    (let* ((kind (repo-tooling-field (quote kind) row))
           (language (repo-tooling-field (quote language) row))
           (lifecycle (repo-tooling-field (quote lifecycle) row))
           (kind-state (repo-tooling-symbol-admission kind repo-tooling-kinds)))
      (cond
        ((eq kind-state (quote admitted)) (identity-relation same)
         (let ((language-state
                 (repo-tooling-symbol-admission language repo-tooling-languages)))
           (cond
             ((eq language-state (quote admitted)) (identity-relation same)
              (let ((lifecycle-state
                      (repo-tooling-symbol-admission lifecycle repo-tooling-lifecycles)))
                (cond
                  ((eq lifecycle-state (quote admitted)) (identity-relation same)
                   (list (quote repo-tooling-ok)))
                  ((eq lifecycle-state (quote rejected)) (identity-relation same)
                   (repo-tooling-violation (quote invalid-lifecycle) lifecycle))
                  ((eq lifecycle-state (quote malformed-admitted-set))
                   (identity-relation same)
                   (repo-tooling-violation
                     (quote malformed-lifecycle-vocabulary)
                     lifecycle)))))
             ((eq language-state (quote rejected)) (identity-relation same)
              (repo-tooling-violation (quote invalid-language) language))
             ((eq language-state (quote malformed-admitted-set))
              (identity-relation same)
              (repo-tooling-violation
                (quote malformed-language-vocabulary)
                language)))))
        ((eq kind-state (quote rejected)) (identity-relation same)
         (repo-tooling-violation (quote invalid-kind) kind))
        ((eq kind-state (quote malformed-admitted-set)) (identity-relation same)
         (repo-tooling-violation (quote malformed-kind-vocabulary) kind))))))

(def repo-tooling-enum-verdict
  (lambda (rows)
    (cond
      ((atom rows) (structural-kind empty-list) (list (quote repo-tooling-ok)))
      ((atom rows) (structural-kind atom)
       (repo-tooling-violation (quote malformed-inventory-list) rows))
      ((atom rows) (structural-kind pair)
       (let ((row-verdict (repo-tooling-row-enum-verdict (car rows))))
         (cond
           ((equal? row-verdict (list (quote repo-tooling-ok)))
            (structural-relation same)
            (repo-tooling-enum-verdict (cdr rows)))
           ((equal? row-verdict (list (quote repo-tooling-ok)))
            (structural-relation distinct)
            row-verdict)))))))

(def repo-tooling-python-migration-required-state
  (lambda (row)
    (let ((language (repo-tooling-field (quote language) row))
          (lifecycle (repo-tooling-field (quote lifecycle) row)))
      (cond
        ((eq language (quote python)) (identity-relation same)
         (cond
           ((eq lifecycle (quote active)) (identity-relation same) (quote required))
           ((eq lifecycle (quote active)) (identity-relation distinct)
            (cond
              ((eq lifecycle (quote transitional))
               (identity-relation same)
               (quote required))
              ((eq lifecycle (quote transitional))
               (identity-relation distinct)
               (quote not-required))))))
        ((eq language (quote python)) (identity-relation distinct)
         (quote not-required))))))

(def repo-tooling-migration-owner-state
  (lambda (owner)
    (cond
      ((atom owner) (structural-kind empty-list) (quote missing))
      ((atom owner) (structural-kind atom)
       (cond
         ((eq owner (quote missing)) (identity-relation same) (quote missing))
         ((eq owner (quote missing)) (identity-relation distinct) (quote present))))
      ((atom owner) (structural-kind pair) (quote present)))))

(def repo-tooling-row-python-migration-verdict
  (lambda (row)
    (let ((required-state (repo-tooling-python-migration-required-state row)))
      (cond
        ((eq required-state (quote not-required)) (identity-relation same)
         (list (quote repo-tooling-ok)))
        ((eq required-state (quote required)) (identity-relation same)
         (let* ((owner (repo-tooling-field (quote migration-issue) row))
                (owner-state (repo-tooling-migration-owner-state owner)))
           (cond
             ((eq owner-state (quote present)) (identity-relation same)
              (list (quote repo-tooling-ok)))
             ((eq owner-state (quote missing)) (identity-relation same)
              (repo-tooling-violation
                (quote python-migration-unowned)
                (repo-tooling-field (quote path) row))))))))))

(def repo-tooling-python-migration-verdict
  (lambda (rows)
    (cond
      ((atom rows) (structural-kind empty-list) (list (quote repo-tooling-ok)))
      ((atom rows) (structural-kind atom)
       (repo-tooling-violation (quote malformed-inventory-list) rows))
      ((atom rows) (structural-kind pair)
       (let ((row-verdict (repo-tooling-row-python-migration-verdict (car rows))))
         (cond
           ((equal? row-verdict (list (quote repo-tooling-ok)))
            (structural-relation same)
            (repo-tooling-python-migration-verdict (cdr rows)))
           ((equal? row-verdict (list (quote repo-tooling-ok)))
            (structural-relation distinct)
            row-verdict)))))))

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

(def repo-tooling-observed-path-state
  (lambda (path observed)
    (cond
      ((atom observed) (structural-kind empty-list) (quote absent))
      ((atom observed) (structural-kind atom) (quote malformed))
      ((atom observed) (structural-kind pair)
       (let ((observed-path (string-append "scripts/" (car observed))))
         (cond
           ((equal? path observed-path) (structural-relation same) (quote present))
           ((equal? path observed-path) (structural-relation distinct)
            (repo-tooling-observed-path-state path (cdr observed)))))))))

(def repo-tooling-stale-path-verdict
  (lambda (rows observed)
    (cond
      ((atom rows) (structural-kind empty-list) (list (quote repo-tooling-ok)))
      ((atom rows) (structural-kind atom)
       (repo-tooling-violation (quote malformed-inventory-list) rows))
      ((atom rows) (structural-kind pair)
       (let* ((row (car rows))
              (path (repo-tooling-field (quote path) row))
              (state (repo-tooling-observed-path-state path observed)))
         (cond
           ((eq state (quote present)) (identity-relation same)
            (repo-tooling-stale-path-verdict (cdr rows) observed))
           ((eq state (quote absent)) (identity-relation same)
            (repo-tooling-violation (quote stale-path) path))
           ((eq state (quote malformed)) (identity-relation same)
            (repo-tooling-violation (quote malformed-observed-list) observed))))))))

(def repo-tooling-verdict-after-migration
  (lambda (rows observed)
    (let ((duplicate-verdict (repo-tooling-duplicate-path-verdict rows)))
      (cond
        ((equal? duplicate-verdict (list (quote repo-tooling-ok)))
         (structural-relation same)
         (let ((stale-verdict (repo-tooling-stale-path-verdict rows observed)))
           (cond
             ((equal? stale-verdict (list (quote repo-tooling-ok)))
              (structural-relation same)
              (repo-tooling-observed-coverage-verdict rows observed))
             ((equal? stale-verdict (list (quote repo-tooling-ok)))
              (structural-relation distinct)
              stale-verdict))))
        ((equal? duplicate-verdict (list (quote repo-tooling-ok)))
         (structural-relation distinct)
         duplicate-verdict)))))

(def repo-tooling-verdict-after-enums
  (lambda (rows observed)
    (let ((migration-verdict (repo-tooling-python-migration-verdict rows)))
      (cond
        ((equal? migration-verdict (list (quote repo-tooling-ok)))
         (structural-relation same)
         (repo-tooling-verdict-after-migration rows observed))
        ((equal? migration-verdict (list (quote repo-tooling-ok)))
         (structural-relation distinct)
         migration-verdict)))))

(def repo-tooling-verdict
  (lambda (rows observed)
    (let ((enum-verdict (repo-tooling-enum-verdict rows)))
      (cond
        ((equal? enum-verdict (list (quote repo-tooling-ok)))
         (structural-relation same)
         (repo-tooling-verdict-after-enums rows observed))
        ((equal? enum-verdict (list (quote repo-tooling-ok)))
         (structural-relation distinct)
         enum-verdict)))))

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

(def repo-tooling-sample-bad-kind
  (quote
    (tool
      (path "scripts/a.lisp")
      (kind mystery-kind)
      (language lisp)
      (role sample)
      (lifecycle active)
      (callers ())
      (authority-source (issue 382))
      (migration-issue ())
      (replacement ())
      (removal-condition ()))))

(def repo-tooling-sample-bad-language
  (quote
    (tool
      (path "scripts/a.lisp")
      (kind check)
      (language mystery-language)
      (role sample)
      (lifecycle active)
      (callers ())
      (authority-source (issue 382))
      (migration-issue ())
      (replacement ())
      (removal-condition ()))))

(def repo-tooling-sample-bad-lifecycle
  (quote
    (tool
      (path "scripts/a.lisp")
      (kind check)
      (language lisp)
      (role sample)
      (lifecycle parity-green)
      (callers ())
      (authority-source (issue 382))
      (migration-issue ())
      (replacement ())
      (removal-condition ()))))

(def repo-tooling-sample-python-unowned
  (quote
    (tool
      (path "scripts/a.py")
      (kind check)
      (language python)
      (role python-without-migration-owner)
      (lifecycle transitional)
      (callers ())
      (authority-source (issue 382))
      (migration-issue ())
      (replacement ())
      (removal-condition parity-green-and-callers-switched))))

(def repo-tooling-selftest-unregistered
  (lambda ()
    (repo-tooling-verdict (list repo-tooling-sample-row-a) (quote ("a.lisp" "b.lisp")))))

(def repo-tooling-selftest-duplicate-path
  (lambda ()
    (repo-tooling-verdict (list repo-tooling-sample-row-a repo-tooling-sample-row-a) (quote ("a.lisp")))))

(def repo-tooling-selftest-stale-path
  (lambda ()
    (repo-tooling-verdict (list repo-tooling-sample-row-a repo-tooling-sample-row-c) (quote ("a.lisp")))))

(def repo-tooling-selftest-invalid-kind
  (lambda ()
    (repo-tooling-verdict (list repo-tooling-sample-bad-kind) (quote ("a.lisp")))))

(def repo-tooling-selftest-invalid-language
  (lambda ()
    (repo-tooling-verdict (list repo-tooling-sample-bad-language) (quote ("a.lisp")))))

(def repo-tooling-selftest-invalid-lifecycle
  (lambda ()
    (repo-tooling-verdict (list repo-tooling-sample-bad-lifecycle) (quote ("a.lisp")))))

(def repo-tooling-selftest-python-unowned
  (lambda ()
    (repo-tooling-verdict (list repo-tooling-sample-python-unowned) (quote ("a.py")))))

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

(repo-tooling-assert-verdict
  (repo-tooling-selftest-stale-path)
  (quote (repo-tooling-violation stale-path "scripts/c.lisp")))

(repo-tooling-assert-verdict
  (repo-tooling-selftest-invalid-kind)
  (quote (repo-tooling-violation invalid-kind mystery-kind)))

(repo-tooling-assert-verdict
  (repo-tooling-selftest-invalid-language)
  (quote (repo-tooling-violation invalid-language mystery-language)))

(repo-tooling-assert-verdict
  (repo-tooling-selftest-invalid-lifecycle)
  (quote (repo-tooling-violation invalid-lifecycle parity-green)))

(repo-tooling-assert-verdict
  (repo-tooling-selftest-python-unowned)
  (quote (repo-tooling-violation python-migration-unowned "scripts/a.py")))

(print
  (quote
    (repo-tooling-selftests-ok
      unregistered-tool duplicate-path stale-path closed-enums python-migration-ownership)))
