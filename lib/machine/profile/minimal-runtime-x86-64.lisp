; #196 — first machine-readable lower-bound profile for one bounded structural Lisp witness.
;
; This file is deliberately NOT a second ISA catalogue, opcode table, encoder,
; or admission list. The observed machine families are derived from the already
; existing semantic lowering for CAR(CONS left right). Hand-authored data below
; explains only why each observed family is necessary for this bounded witness.
;
; Current claim boundary:
;   (car (cons 2 3)) -> 2
; using one host-provided 16-byte pair arena whose lifetime is one native call.
; This is not a full allocator, GC, calling convention, or self-hosting claim.

(def x86-minimal-structural-car-dependencies
  (quote
    ((materialize-value
       mov-r64-imm64
       required-for-minimal-runtime
       "materialize bounded u64 pair fields")
     (store-pair-field
       mov-mem-disp8-r64
       required-for-minimal-runtime
       "store bounded pair head/tail into the host-provided arena")
     (load-pair-field
       mov-r64-mem-disp8
       required-for-minimal-runtime
       "load the CAR field from the bounded pair")
     (return-result
       ret
       required-for-minimal-runtime
       "return the bounded result through the host ABI"))))

(def x86-minimal-family-member?
  (lambda (family families)
    (cond
      ((atom families) ())
      ((eq family (car families)) t)
      (t (x86-minimal-family-member? family (cdr families))))))

(def x86-minimal-unique-form-families
  (lambda (forms seen)
    (cond
      ((atom forms) seen)
      (t
       (let ((family (car (car forms))))
         (cond
           ((x86-minimal-family-member? family seen)
            (x86-minimal-unique-form-families (cdr forms) seen))
           (t
            (x86-minimal-unique-form-families
              (cdr forms)
              (append seen (list family))))))))))

(def x86-minimal-row-second
  (lambda (row)
    (car (cdr row))))

(def x86-minimal-row-third
  (lambda (row)
    (car (cdr (cdr row)))))

(def x86-minimal-map-row-second
  (lambda (rows)
    (cond
      ((atom rows) (quote ()))
      (t
       (cons
         (x86-minimal-row-second (car rows))
         (x86-minimal-map-row-second (cdr rows)))))))

(def x86-minimal-map-row-third
  (lambda (rows)
    (cond
      ((atom rows) (quote ()))
      (t
       (cons
         (x86-minimal-row-third (car rows))
         (x86-minimal-map-row-third (cdr rows)))))))

(def x86-minimal-structural-car-forms
  (lambda (left right)
    (x86-lower-cons-car-u64-forms left right)))

(def x86-minimal-structural-car-observed-families
  (lambda (left right)
    (x86-minimal-unique-form-families
      (x86-minimal-structural-car-forms left right)
      (quote ()))))

(def x86-minimal-structural-car-dependency-families
  (lambda ()
    (x86-minimal-map-row-second x86-minimal-structural-car-dependencies)))

(def x86-minimal-structural-car-dependency-classes
  (lambda ()
    (x86-minimal-map-row-third x86-minimal-structural-car-dependencies)))

(def x86-minimal-structural-car-profile
  (lambda (left right)
    (list
      (list (quote witness) (quote bounded-car-cons-u64))
      (list (quote forms) (x86-minimal-structural-car-forms left right))
      (list
        (quote observed-families)
        (x86-minimal-structural-car-observed-families left right))
      (list (quote dependencies) x86-minimal-structural-car-dependencies)
      (list (quote arena-lifetime) (quote native-call))
      (list (quote escape) (quote forbidden))
      (list (quote claim) (quote bounded-structural-lower-bound)))))
