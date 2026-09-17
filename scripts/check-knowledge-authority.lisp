; #383 — Lisp-owned knowledge artifact authority classification checker.
; GREEN slices:
; - newly observed artifacts require explicit classification rows;
; - every classification row must carry the required provenance fields;
; - directory placement is never an authority source;
; - registry rows must still exist in the observed knowledge tree.

(def knowledge-authority-required-fields
  (quote (path class scope authority-source lifecycle consumers)))

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

(def knowledge-authority-verdict-ok-state
  (lambda (verdict)
    (cond
      ((equal? verdict (list (quote knowledge-authority-ok)))
       (structural-relation same)
       (quote yes))
      ((equal? verdict (list (quote knowledge-authority-ok)))
       (structural-relation distinct)
       (quote no)))))

(def knowledge-authority-required-fields-verdict
  (lambda (required row)
    (cond
      ((atom required) (structural-kind empty-list)
       (list (quote knowledge-authority-ok)))
      ((atom required) (structural-kind atom)
       (knowledge-authority-violation (quote malformed-required-field-list) required))
      ((atom required) (structural-kind pair)
       (let* ((name (car required))
              (value (knowledge-authority-field name row)))
         (cond
           ((equal? value (quote missing)) (structural-relation same)
            (knowledge-authority-violation (quote missing-field) name))
           ((equal? value (quote missing)) (structural-relation distinct)
            (knowledge-authority-required-fields-verdict (cdr required) row))))))))

(def knowledge-authority-row-required-verdict
  (lambda (row)
    (cond
      ((atom row) (structural-kind empty-list)
       (knowledge-authority-violation (quote malformed-row) row))
      ((atom row) (structural-kind atom)
       (knowledge-authority-violation (quote malformed-row) row))
      ((atom row) (structural-kind pair)
       (cond
         ((eq (car row) (quote artifact)) (identity-relation same)
          (knowledge-authority-required-fields-verdict
            knowledge-authority-required-fields
            row))
         ((eq (car row) (quote artifact)) (identity-relation distinct)
          (knowledge-authority-violation (quote malformed-row-kind) (car row))))))))

(def knowledge-authority-required-verdict
  (lambda (rows)
    (cond
      ((atom rows) (structural-kind empty-list)
       (list (quote knowledge-authority-ok)))
      ((atom rows) (structural-kind atom)
       (knowledge-authority-violation (quote malformed-inventory-list) rows))
      ((atom rows) (structural-kind pair)
       (let ((row-verdict (knowledge-authority-row-required-verdict (car rows))))
         (cond
           ((eq (knowledge-authority-verdict-ok-state row-verdict) (quote yes))
            (identity-relation same)
            (knowledge-authority-required-verdict (cdr rows)))
           ((eq (knowledge-authority-verdict-ok-state row-verdict) (quote no))
            (identity-relation same)
            row-verdict)))))))

(def knowledge-authority-directory-source-state
  (lambda (source)
    (cond
      ((atom source) (structural-kind empty-list) (quote no))
      ((atom source) (structural-kind atom) (quote no))
      ((atom source) (structural-kind pair)
       (cond
         ((eq (car source) (quote directory)) (identity-relation same) (quote yes))
         ((eq (car source) (quote directory)) (identity-relation distinct) (quote no)))))))

(def knowledge-authority-row-source-verdict
  (lambda (row)
    (let* ((source (knowledge-authority-field (quote authority-source) row))
           (directory-state (knowledge-authority-directory-source-state source)))
      (cond
        ((eq directory-state (quote yes)) (identity-relation same)
         (knowledge-authority-violation
           (quote directory-derived-authority)
           (knowledge-authority-field (quote path) row)))
        ((eq directory-state (quote no)) (identity-relation same)
         (list (quote knowledge-authority-ok)))))))

(def knowledge-authority-source-verdict
  (lambda (rows)
    (cond
      ((atom rows) (structural-kind empty-list)
       (list (quote knowledge-authority-ok)))
      ((atom rows) (structural-kind atom)
       (knowledge-authority-violation (quote malformed-inventory-list) rows))
      ((atom rows) (structural-kind pair)
       (let ((row-verdict (knowledge-authority-row-source-verdict (car rows))))
         (cond
           ((eq (knowledge-authority-verdict-ok-state row-verdict) (quote yes))
            (identity-relation same)
            (knowledge-authority-source-verdict (cdr rows)))
           ((eq (knowledge-authority-verdict-ok-state row-verdict) (quote no))
            (identity-relation same)
            row-verdict)))))))

(def knowledge-authority-observed-path-state
  (lambda (path observed)
    (cond
      ((atom observed) (structural-kind empty-list) (quote missing))
      ((atom observed) (structural-kind atom) (quote malformed))
      ((atom observed) (structural-kind pair)
       (let ((observed-path (string-append "knowledge/" (car observed))))
         (cond
           ((equal? path observed-path) (structural-relation same) (quote present))
           ((equal? path observed-path) (structural-relation distinct)
            (knowledge-authority-observed-path-state path (cdr observed)))))))))

(def knowledge-authority-stale-path-verdict
  (lambda (rows observed)
    (cond
      ((atom rows) (structural-kind empty-list)
       (list (quote knowledge-authority-ok)))
      ((atom rows) (structural-kind atom)
       (knowledge-authority-violation (quote malformed-inventory-list) rows))
      ((atom rows) (structural-kind pair)
       (let* ((row (car rows))
              (path (knowledge-authority-field (quote path) row))
              (path-state (knowledge-authority-observed-path-state path observed)))
         (cond
           ((eq path-state (quote present)) (identity-relation same)
            (knowledge-authority-stale-path-verdict (cdr rows) observed))
           ((eq path-state (quote missing)) (identity-relation same)
            (knowledge-authority-violation (quote stale-path) path))
           ((eq path-state (quote malformed)) (identity-relation same)
            (knowledge-authority-violation (quote malformed-observed-list) observed))))))))

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

(def knowledge-authority-verdict-after-source
  (lambda (rows observed)
    (let ((stale-verdict (knowledge-authority-stale-path-verdict rows observed)))
      (cond
        ((eq (knowledge-authority-verdict-ok-state stale-verdict) (quote yes))
         (identity-relation same)
         (knowledge-authority-observed-coverage-verdict rows observed))
        ((eq (knowledge-authority-verdict-ok-state stale-verdict) (quote no))
         (identity-relation same)
         stale-verdict)))))

(def knowledge-authority-verdict-after-required
  (lambda (rows observed)
    (let ((source-verdict (knowledge-authority-source-verdict rows)))
      (cond
        ((eq (knowledge-authority-verdict-ok-state source-verdict) (quote yes))
         (identity-relation same)
         (knowledge-authority-verdict-after-source rows observed))
        ((eq (knowledge-authority-verdict-ok-state source-verdict) (quote no))
         (identity-relation same)
         source-verdict)))))

(def knowledge-authority-verdict
  (lambda (rows observed)
    (let ((required-verdict (knowledge-authority-required-verdict rows)))
      (cond
        ((eq (knowledge-authority-verdict-ok-state required-verdict) (quote yes))
         (identity-relation same)
         (knowledge-authority-verdict-after-required rows observed))
        ((eq (knowledge-authority-verdict-ok-state required-verdict) (quote no))
         (identity-relation same)
         required-verdict)))))

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

(def knowledge-authority-sample-directory-authority
  (quote
    (artifact
      (path "knowledge/a.lisp")
      (class operational-reference)
      (scope sample)
      (authority-source (directory "knowledge/"))
      (lifecycle active)
      (consumers ()))))

(def knowledge-authority-sample-stale-row
  (quote
    (artifact
      (path "knowledge/c.lisp")
      (class operational-reference)
      (scope stale-sample)
      (authority-source (issue 383))
      (lifecycle active)
      (consumers ()))))

(def knowledge-authority-selftest-unclassified
  (lambda ()
    (knowledge-authority-verdict
      (list knowledge-authority-sample-row)
      (quote ("a.lisp" "b.lisp")))))

(def knowledge-authority-selftest-missing-class
  (lambda ()
    (knowledge-authority-verdict
      (list knowledge-authority-sample-missing-class)
      (quote ("a.lisp")))))

(def knowledge-authority-selftest-directory-authority
  (lambda ()
    (knowledge-authority-verdict
      (list knowledge-authority-sample-directory-authority)
      (quote ("a.lisp")))))

(def knowledge-authority-selftest-stale-path
  (lambda ()
    (knowledge-authority-verdict
      (list knowledge-authority-sample-row knowledge-authority-sample-stale-row)
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

(knowledge-authority-assert-verdict
  (knowledge-authority-selftest-directory-authority)
  (quote
    (knowledge-authority-violation
      directory-derived-authority
      "knowledge/a.lisp")))

(knowledge-authority-assert-verdict
  (knowledge-authority-selftest-stale-path)
  (quote
    (knowledge-authority-violation
      stale-path
      "knowledge/c.lisp")))

(print
  (quote
    (knowledge-authority-selftests-ok
      unclassified-artifact
      missing-field
      directory-derived-authority
      stale-path)))
