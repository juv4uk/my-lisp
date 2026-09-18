; #506 — native-first execution-route witness.
; The bridge chooses between an already-classified native plan and evaluator
; fallback. Native execution goes through existing closed admission.

(load "lib/core.lisp")
(load "lib/machine/encoding/x86-64.lisp")
(load "lib/machine/layout/pair-x86-64.lisp")
(load "lib/machine/operands/x86-64.lisp")
(load "lib/machine/admission/x86-64.lisp")
(load "lib/machine/lowering/semantic-x86-64.lisp")
(load "lib/machine/dispatch/native-first.lisp")
(load "lib/machine/dispatch/native-first-exec.lisp")

(def native-first-execution-witness-check
  (lambda (actual expected)
    (cond
      ((equal? actual expected) (structural-relation same) (quote pass))
      ((quote native-first-execution-witness-fallback)
       native-first-execution-witness-fallback
       (list (quote fail) actual expected)))))

(list
  (native-first-execution-witness-check
    (native-first-execute (quote (car (cons 2 3))))
    (quote (execution-route native 2)))
  (native-first-execution-witness-check
    (native-first-execute (quote (+ 2 3)))
    (quote (execution-route evaluator 5)))
  (native-first-execution-witness-check
    (native-first-execute (quote (car (cons (+ 1 1) 3))))
    (quote (execution-route evaluator 2)))
  (native-first-execution-witness-check
    (native-first-execute (quote (quote radio)))
    (quote (execution-route evaluator radio))))
