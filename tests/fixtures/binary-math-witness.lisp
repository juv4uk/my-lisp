; #216 — Lisp-owned verdict for exact-Q binary comparison rows.
; Uses #217 explicit three-part cond and #218 structural result matching.

(def binary-field
  (lambda (row field)
    (let ((found (assoc field row)))
      (cond
        ((atom found) (structural-kind pair) (cdr found))
        ((quote binary-field-fallback) binary-field-fallback (quote ()))))))

(def binary-math-result
  (lambda (status decision expected-kind actual-kind expected-render actual-render)
    (list (quote binary-math-result)
          (list (quote status) status)
          (list (quote decision) decision)
          (list (quote expected-kind) expected-kind)
          (list (quote actual-kind) actual-kind)
          (list (quote expected-render) expected-render)
          (list (quote actual-render) actual-render))))

(def binary-math-verdict
  (lambda (row actual-kind actual-render)
    (let ((expected-kind (binary-field row (quote expected-kind)))
          (expected-render (binary-field row (quote expected-render)))
          (decision (binary-field row (quote decision))))
      (cond
        ((equal? expected-kind actual-kind) (structural-relation same)
         (cond
           ((equal? expected-render actual-render) (structural-relation same)
            (binary-math-result (quote pass)
                                decision
                                expected-kind
                                actual-kind
                                expected-render
                                actual-render))
           ((quote binary-render-fallback) binary-render-fallback
            (binary-math-result (quote fail)
                                decision
                                expected-kind
                                actual-kind
                                expected-render
                                actual-render))))
        ((quote binary-kind-fallback) binary-kind-fallback
         (binary-math-result (quote fail)
                             decision
                             expected-kind
                             actual-kind
                             expected-render
                             actual-render))))))

(def binary-math-status
  (lambda (result)
    (second (second result))))
