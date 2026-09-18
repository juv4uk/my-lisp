; Function genealogy experiment.
; The semantic registry supplies opaque BitPattern8 identities.  This file asks
; a deliberately empirical question: which registered operations are sufficient
; to reconstruct other registered operations using ordinary Lisp?
;
; This is NOT a new authority table.  Edges below are experimental evidence
; backed by executable redefinitions in this file.
;
; Registry identities used here:
;   quote   00000001
;   cons    00000100
;   car     00000101
;   cdr     00000110
;   pair    00101110
;   second  00101111
;   third   00110000
;   fourth  00110001
;   fifth   00110010
;   caar    00110011
;   cadr    00110100
;   cddr    00110101

(def genealogy-depends-on 11110000)

; One edge means: SOURCE is directly used by an executable laboratory
; reconstruction of TARGET.  Relation meaning lives in this experiment, not in
; the bit pattern itself.
(def function-genealogy
  (list
    (list 00000100 genealogy-depends-on 00101110)
    (list 00000001 genealogy-depends-on 00101110)

    (list 00000101 genealogy-depends-on 00101111)
    (list 00000110 genealogy-depends-on 00101111)

    (list 00000101 genealogy-depends-on 00110000)
    (list 00000110 genealogy-depends-on 00110000)

    (list 00000101 genealogy-depends-on 00110001)
    (list 00000110 genealogy-depends-on 00110001)

    (list 00000101 genealogy-depends-on 00110010)
    (list 00000110 genealogy-depends-on 00110010)

    (list 00000101 genealogy-depends-on 00110011)

    (list 00000101 genealogy-depends-on 00110100)
    (list 00000110 genealogy-depends-on 00110100)

    (list 00000110 genealogy-depends-on 00110101)))

; Laboratory reconstructions.  The names are intentionally not registry names:
; the existing functions remain untouched while we test sufficiency.

(def experiment-pair
  (lambda (left right)
    (cons left (cons right (quote ())))))

(def experiment-second
  (lambda (values)
    (car (cdr values))))

(def experiment-third
  (lambda (values)
    (car (cdr (cdr values)))))

(def experiment-fourth
  (lambda (values)
    (car (cdr (cdr (cdr values))))))

(def experiment-fifth
  (lambda (values)
    (car (cdr (cdr (cdr (cdr values)))))))

(def experiment-caar
  (lambda (values)
    (car (car values))))

(def experiment-cadr
  (lambda (values)
    (car (cdr values))))

(def experiment-cddr
  (lambda (values)
    (cdr (cdr values))))

; Explicit result records keep () available as ordinary data.
(def genealogy-check
  (lambda (identity observed expected)
    (cond
      ((equal? observed expected) (structural-relation same)
       (list identity (quote reproduced)))
      ((equal? observed expected) (structural-relation distinct)
       (list identity (quote diverged))))))

(def function-genealogy-witness
  (lambda ()
    (let ((flat (quote (a b c d e))))
      (let ((nested (quote ((left inner) right))))
        (list
          (genealogy-check
            00101110
            (experiment-pair (quote left) (quote right))
            (pair (quote left) (quote right)))
          (genealogy-check 00101111 (experiment-second flat) (second flat))
          (genealogy-check 00110000 (experiment-third flat) (third flat))
          (genealogy-check 00110001 (experiment-fourth flat) (fourth flat))
          (genealogy-check 00110010 (experiment-fifth flat) (fifth flat))
          (genealogy-check 00110011 (experiment-caar nested) (caar nested))
          (genealogy-check 00110100 (experiment-cadr flat) (cadr flat))
          (genealogy-check 00110101 (experiment-cddr flat) (cddr flat)))))))

; Graph queries over the experimental genealogy.  They are directed:
; "what targets directly depend on SOURCE?"  We intentionally return every
; matching target rather than choosing one.

(def genealogy-direct-targets
  (lambda (graph source)
    (cond
      ((atom graph) (structural-kind empty-list)
       (quote ()))
      ((atom graph) (structural-kind pair)
       (let ((edge (car graph)))
         (cond
           ((eq (car edge) source) (identity-relation same)
            (cons
              (third edge)
              (genealogy-direct-targets (cdr graph) source)))
           ((eq (car edge) source) (identity-relation distinct)
            (genealogy-direct-targets (cdr graph) source)))))
      ((atom graph) (structural-kind atom)
       (quote ())))))

(def function-genealogy-observation
  (lambda ()
    (list
      (list
        (quote from-cons)
        (genealogy-direct-targets function-genealogy 00000100))
      (list
        (quote from-car)
        (genealogy-direct-targets function-genealogy 00000101))
      (list
        (quote from-cdr)
        (genealogy-direct-targets function-genealogy 00000110))
      (list
        (quote reproductions)
        (function-genealogy-witness)))))
