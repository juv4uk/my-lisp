; #506 — Lisp-owned native-first execution bridge.
;
; Routing policy remains in Lisp:
;   native-plan       -> closed admission -> host execution mechanism -> CPU
;   evaluator-fallback -> eval of the unchanged original expression
;
; There is deliberately no catch-and-fallback around a native plan. Once the
; classifier admits a native route, an admission/host/machine failure must
; remain visible instead of being hidden by reference-evaluator success.

(def native-first-execution-result
  (lambda (route value)
    (list (quote execution-route) route value)))

(def native-first-execute-plan
  (lambda (plan)
    (cond
      ((eq (car plan) (quote native-plan)) (identity-relation same)
       (native-first-execution-result
         (quote native)
         (x86-call-admitted-u64 (second plan) (third plan))))
      ((eq (car plan) (quote evaluator-fallback)) (identity-relation same)
       (native-first-execution-result
         (quote evaluator)
         (eval (second plan))))
      ((quote native-first-invalid-plan)
       native-first-invalid-plan
       (list (quote invalid-native-plan) plan)))))

(def native-first-execute
  (lambda (expression)
    (native-first-execute-plan (native-first-plan expression))))
