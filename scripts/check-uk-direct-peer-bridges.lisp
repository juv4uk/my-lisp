; Ukrainian direct-peer topology guard.
; Охоронець прямих українських peer-поверхонь.
;
; This is mechanism/topology evidence, not semantic authority. The numeric
; semantic registry already owns which spellings share identity. This guard
; only prevents a migrated UK spelling from quietly returning to the old
; `(define UK EN)` bridge topology.
;
; Keep this guard deliberately narrow while answer-semantics migration is in
; flight: it avoids list predicates that still depend on historical atom/NIL
; truthiness and checks only already-migrated source topology.
;
; Run from the repository root with the normal CLI host capabilities:
;   ./target/debug/my-lisp scripts/check-uk-direct-peer-bridges.lisp

(def uk-source (read-file "lib/surface/uk.lisp"))
(def old-1043-bridge "(define зчепити string-append)")

(cond
  ((string-contains? old-1043-bridge uk-source) t
   ((lambda (printed)
      (car (quote ())))
    (print
      (quote
        (uk-direct-peer-bridge-violation
          (semantic-id 1043)
          (uk зчепити)
          (en string-append))))))
  (t t
   (print
     (quote
       (uk-direct-peer-bridge-check
         (status pass)
         (ids (1043)))))))
