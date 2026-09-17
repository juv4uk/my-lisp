; #369 — preserve public CLIPS importer behavior across the symbol? result-domain
; migration without preserving the historical t/() predicate sentinel.

(load "lib/clips-import.lisp")

(def clips-symbol-boundary-rows
  (lambda ()
    (list
      (list
        (quote qualified-template-name-normalizes)
        (clips-strip-module-prefix (quote QUESTIONS::question))
        (quote question))
      (list
        (quote non-symbol-template-name-passes-through)
        (clips-strip-module-prefix 42)
        42)
      (list
        (quote clips-variable-converts-to-logic-var)
        (clips-convert-vars (quote ?x))
        (quote (var x)))
      (list
        (quote non-symbol-term-passes-through)
        (clips-convert-vars 98)
        98))))

(def clips-symbol-boundary-check
  (lambda (rows)
    (cond
      ((atom rows) (structural-kind empty-list)
       (quote (clips-symbol-boundary-witness (status pass))))
      ((atom rows) (structural-kind atom)
       (list
         (quote clips-symbol-boundary-witness)
         (list (quote status) (quote fail))
         (list (quote case) (quote malformed-row-tail))
         (list (quote actual) rows)))
      ((atom rows) (structural-kind pair)
       (let ((row (car rows)))
         (cond
           ((equal? (second row) (third row)) (structural-relation same)
            (clips-symbol-boundary-check (cdr rows)))
           ((equal? (second row) (third row)) (structural-relation distinct)
            (list
              (quote clips-symbol-boundary-witness)
              (list (quote status) (quote fail))
              (list (quote case) (car row))
              (list (quote actual) (second row))
              (list (quote expected) (third row))))))))))

(def clips-symbol-boundary-witness
  (lambda ()
    (clips-symbol-boundary-check (clips-symbol-boundary-rows))))

(clips-symbol-boundary-witness)
