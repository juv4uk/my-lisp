; #115 — Lisp owns the authority verdict.
; Usage: CI writes change facts as Lisp data to tests/changed-host-tests.lisp,
; then redirects this program's final value to tests/authority-verdict.lisp.
; Host code transports paths/diff direction only; it never decides authority.
;
; Change rows:
;   (change "path" deletion-only)
;   (change "path" modified)
;
; Verdicts:
;   (authority-ok)
;   (semantic-authority-violation "path" class "diagnostic")
;
; This producer never intentionally fails.  A second Lisp process,
; scripts/authority-guard-enforce.lisp, owns fail-closed enforcement so the
; verdict remains visible even when the enforcer exits non-zero.

(def second (lambda (x) (car (cdr x))))
(def third (lambda (x) (car (cdr (cdr x)))))

(def authority-text (read-file "tests/authority-inventory.lisp"))
(print (quote DEBUG-AFTER-AUTHORITY-READ))
(def authority-rows (read-all authority-text))
(print (quote DEBUG-AFTER-AUTHORITY-PARSE))
(def changed-host-text (read-file "tests/changed-host-tests.lisp"))
(print (quote DEBUG-AFTER-CHANGES-READ))
(def changed-host-tests (read-all changed-host-text))
(print (quote DEBUG-AFTER-CHANGES-PARSE))

(def find-authority
  (lambda (path rows)
    (cond
      ((atom rows) (structural-kind empty-list)
       (quote unclassified))
      ((atom rows) (structural-kind pair)
       (cond
         ((equal? path (second (car rows))) (structural-relation same)
          (third (car rows)))
         ((equal? path (second (car rows))) (structural-relation distinct)
          (find-authority path (cdr rows))))))))

(def allowed-authority?
  (lambda (class)
    (cond
      ((eq class (quote observer)) (identity-relation same)
       (quote allowed))
      ((eq class (quote observer)) (identity-relation distinct)
       (cond
         ((eq class (quote mechanism)) (identity-relation same)
          (quote allowed))
         ((eq class (quote mechanism)) (identity-relation distinct)
          (quote denied)))))))

(def authority-verdict-for
  (lambda (changes)
    (cond
      ((atom changes) (structural-kind empty-list)
       (quote (authority-ok)))
      ((atom changes) (structural-kind pair)
       (let ((change (car changes)))
         (let ((path (second change)))
           (let ((direction (third change)))
             (cond
               ; One-way migration valve: deleting host semantic authority can
               ; only reduce authority, so it may pass regardless of its old
               ; inventory class.  This is not a new allowed authority class.
               ((eq direction (quote deletion-only)) (identity-relation same)
                (authority-verdict-for (cdr changes)))
               ((eq direction (quote deletion-only)) (identity-relation distinct)
                (let ((class (find-authority path authority-rows)))
                  (let ((permission (allowed-authority? class)))
                    (cond
                      ((eq permission (quote allowed)) (identity-relation same)
                       (authority-verdict-for (cdr changes)))
                      ((eq permission (quote allowed)) (identity-relation distinct)
                       (list
                         (quote semantic-authority-violation)
                         path
                         class
                         "Host tests may observe mechanism; Lisp owns meaning. See #112/#113."))))))))))))))

(print (quote DEBUG-BEFORE-EMPTY))
(print (authority-verdict-for (quote ())))
(print (quote DEBUG-BEFORE-ONE))
(print (car changed-host-tests))
(print (authority-verdict-for (cons (car changed-host-tests) (quote ()))))
(print (quote DEBUG-FINISH))
