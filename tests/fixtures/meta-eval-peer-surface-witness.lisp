; #329 / #220 — meta-evaluator peer-surface parity is Lisp-owned authority.
; Preserve the useful law from the retired Rust oracle without restoring its
; historical universal `t` expectations. Canon atom peers now expect the
; explicit structural result `(structural-kind atom)`; necessary-form peers
; retain their current program results. The shell observes only the named
; pass envelope.

(load "lib/meta-eval.lisp")

(def meta-eval-peer-surface-cases
  (quote
    ((expr atom-en
       (atom (quote x))
       (structural-kind atom))
     (expr atom-uk
       (атом? (як-є x))
       (structural-kind atom))
     (expr atom-sa
       (aṇu (svarūpa x))
       (structural-kind atom))
     (expr atom-sym
       (.? (quote x))
       (structural-kind atom))
     (expr lambda-en
       ((lambda (x) x) 42)
       42)
     (expr lambda-uk
       ((функція (x) x) 42)
       42)
     (program define-en
       ((define answer-en 42))
       42)
     (program define-uk
       ((визначити answer-uk 42))
       42)
     (program def-compat
       ((def answer-compat 42))
       42))))

(def meta-eval-peer-surface-actual
  (lambda (mode form)
    (cond
      ((eq mode (quote expr)) (identity-relation same)
       (my-eval form (quote ())))
      ((eq mode (quote program)) (identity-relation same)
       (cdr (my-eval-program form (quote ()))))
      (t t
       (list (quote invalid-witness-mode) mode)))))

(def meta-eval-peer-surface-check
  (lambda (cases)
    (cond
      ((equal? cases (quote ())) (structural-relation same)
       (quote (meta-eval-peer-surface-witness (status pass))))
      ((equal? cases (quote ())) (structural-relation distinct)
       (let ((case (car cases)))
         (let ((mode (car case))
               (law (second case))
               (form (third case))
               (expected (fourth case)))
           (let ((actual (meta-eval-peer-surface-actual mode form)))
             (cond
               ((equal? actual expected) (structural-relation same)
                (meta-eval-peer-surface-check (cdr cases)))
               ((equal? actual expected) (structural-relation distinct)
                (list
                  (quote meta-eval-peer-surface-witness)
                  (quote (status fail))
                  (list (quote law) law)
                  (list (quote expected) expected)
                  (list (quote actual) actual)))))))))))

(meta-eval-peer-surface-check meta-eval-peer-surface-cases)
