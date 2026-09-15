; #115 — Lisp owns the authority verdict.
; Usage: my-lisp scripts/authority-guard.lisp after CI writes changed paths as
; Lisp data to tests/changed-host-tests.lisp. Host code only transports paths
; and observes this program's exit status.

(def second (lambda (x) (car (cdr x))))
(def third (lambda (x) (car (cdr (cdr x)))))

(def authority-rows (read-all (read-file "tests/authority-inventory.lisp")))
(def changed-paths (read-all (read-file "tests/changed-host-tests.lisp")))

(def find-authority
  (lambda (path rows)
    (cond
      ((atom rows) ())
      ((equal? path (second (car rows))) (third (car rows)))
      (t (find-authority path (cdr rows))))))

(def allowed-authority?
  (lambda (class)
    (cond
      ((eq class (quote observer)) t)
      ((eq class (quote mechanism)) t)
      (t ()))))

(def fail-authority
  (lambda (path class)
    (cons
      (print (list (quote semantic-authority-violation)
                   path class
                   "Host tests may observe mechanism; Lisp owns meaning. See #112/#113."))
      (car ()))))

(def check-paths
  (lambda (paths)
    (cond
      ((atom paths) t)
      (t
       (let ((class (find-authority (car paths) authority-rows)))
         (cond
           ((allowed-authority? class) (check-paths (cdr paths)))
           (t (fail-authority (car paths) class))))))))

(check-paths changed-paths)
