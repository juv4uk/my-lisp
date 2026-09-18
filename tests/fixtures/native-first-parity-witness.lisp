; #509 — native/evaluator differential parity gate for every currently
; admitted native-first island. The first classifier island is bounded
; CAR(CONS <u64> <u64>). Expected answers are independent Lisp-owned evidence;
; neither the native route nor the evaluator is allowed to be its own oracle.

(load "lib/core.lisp")
(load "lib/machine/encoding/x86-64.lisp")
(load "lib/machine/layout/pair-x86-64.lisp")
(load "lib/machine/operands/x86-64.lisp")
(load "lib/machine/admission/x86-64.lisp")
(load "lib/machine/lowering/semantic-x86-64.lisp")
(load "lib/machine/dispatch/native-first.lisp")
(load "lib/machine/dispatch/native-first-execute.lisp")
(load "lib/machine/dispatch/native-first-parity.lisp")

(def native-first-parity-corpus
  (quote
    ((car-cons-u64-zero
       (car (cons 0 1))
       0
       pure
       not-applicable)
     (car-cons-u64-small
       (car (cons 2 3))
       2
       pure
       not-applicable)
     (car-cons-u64-independent-fields
       (car (cons 42 99))
       42
       pure
       not-applicable))))

(def native-first-parity-verdicts
  (native-first-parity-run native-first-parity-corpus))

(def native-first-parity-witness
  (lambda ()
    (cond
      ((equal? (native-first-parity-all-pass? native-first-parity-verdicts) t)
       (structural-relation same)
       (list
         (quote native-first-parity-witness)
         (quote (status pass))
         (list (quote cases) (length native-first-parity-corpus))))
      ((equal? (native-first-parity-all-pass? native-first-parity-verdicts) t)
       (structural-relation distinct)
       (list
         (quote native-first-parity-witness)
         (quote (status fail))
         (list (quote verdicts) native-first-parity-verdicts))))))

(native-first-parity-witness)
