; Ukrainian direct-peer topology guard.
; Охоронець прямих українських peer-поверхонь.
;
; This is mechanism/topology evidence, not semantic authority. The numeric
; semantic registry already owns which spellings share identity. This guard
; only prevents a migrated UK spelling from quietly returning to the old
; `(define UK EN)` bridge topology.
;
; Run from the repository root with the normal CLI host capabilities:
;   ./target/debug/my-lisp scripts/check-uk-direct-peer-bridges.lisp

(def uk-forms (read-all (read-file "lib/surface/uk.lisp")))

; Keep this list narrow: add a row only after that semantic identity has a
; registry-driven direct peer path. Row schema:
;   (semantic-id forbidden-top-level-form)
(def migrated-direct-peers
  (quote
    ((1043 (define зчепити string-append)))))

(def second
  (lambda (value) (car (cdr value))))

(def bridge-present?
  (lambda (row)
    (member? (second row) uk-forms)))

(def first-violation
  (lambda (rows)
    (cond
      ((atom rows) (structural-kind empty-list) (quote ()))
      ((bridge-present? (car rows)) t (car rows))
      (t t (first-violation (cdr rows))))))

(def violation (first-violation migrated-direct-peers))

(cond
  ((atom violation) (structural-kind empty-list)
   (print (quote (uk-direct-peer-bridge-check (status pass) (ids (1043))))))
  (t t
   (cons
     (print (list (quote uk-direct-peer-bridge-violation) violation))
     (car (quote ())))))
