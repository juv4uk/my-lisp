; RED contract for #115.
; Semantic authority policy belongs to Lisp. Host tooling may transport paths,
; inventory rows and exit status, but it must not decide semantic authority.
;
; TECH-SCAN: follows the portable Scheme/SRFI-64 separation where tests carry
; expectations and runners execute/report them; policy is ordinary Lisp data.

(def authority-class-allowed?
  (lambda (class)
    (cond
      ((eq class (quote mechanism)) t)
      ((eq class (quote observer)) t)
      (t (quote ())))))

(def authority-class-forbidden?
  (lambda (class)
    (cond
      ((eq class (quote semantic-authority)) t)
      ((eq class (quote mixed)) t)
      (t (quote ())))))

(def authority-verdict
  (lambda (class)
    (cond
      ((authority-class-allowed? class) (quote allow))
      ((authority-class-forbidden? class) (quote reject))
      (t (quote reject)))))
