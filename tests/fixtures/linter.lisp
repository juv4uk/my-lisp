(
  (expr . "(lint-size 5)")
  (expected . "1")
)
(
  (expr . "(lint-size (quote (a b)))")
  (expected . "5")
)
(
  (expr . "(lint-nesting (quote (a (b c) d)))")
  (expected . "2") ; (b c) is nested 1 level inside the main list... wait. A list of depth 1 has max nesting 1?
  ; '(a) is (a . ()). Nesting(a)=0, Nesting(())=0. Nesting((a)) = max(1+0, 0) = 1.
  ; '(a b) = (a . (b . ())). max(1+0, 1) = 1.
  ; '(a (b c) d) = (a . ((b c) . (d . ()))).
  ; (b c) has nesting 1. (b c) is the car of ((b c) . (d)). So 1+1 = 2! Yes, 2!
)
(
  (expr . "(lint-complexity (quote (cond (a b) (c d))))")
  (expected . "2")
)
(
  (expr . "(lint-complexity (quote (def f (lambda (x) (cond (x 1) (t 2))))))")
  (expected . "2")
)
(
  (expr . "(lint-effects (quote (def f (lambda () (print \"hi\")))))")
  (expected . "((print \"hi\"))")
)
(
  (expr . "(lint-globals (quote (lambda (x y) (+ x y z))))")
  (expected . "(z)")
)
(
  (expr . "(lint-globals (quote (let* ((x 1) (y x)) (+ x y z))))")
  (expected . "(z)")
)
(
  (expr . "(lint-globals (quote (letrec ((even? (lambda (n) (odd? n))) (odd? (lambda (n) (even? n)))) (even? z))))")
  (expected . "(z)")
)
(
  (expr . "(lint-globals (quote (let loop ((n 10)) (cond ((= n 0) z) (t (loop (- n 1)))))))")
  (expected . "(z)")
)
(
  (expr . "(lint-globals (quote (defmacro m (a b) (+ a b c))))")
  (expected . "(c)")
)
(
  (expr . "(get-threshold (quote max-size) (quote ((max-size . 50))) 99)")
  (expected . "50")
)
(
  (expr . "(get-threshold (quote max-nesting) (quote ((max-size . 50))) 99)")
  (expected . "99")
)
(
  (expr . "(lint-check (quote (def f (lambda (x y) (cond (x 1) (t 2))))) (quote ((max-complexity . 1) (max-nesting . 10))))")
  (expected . "((complexity-exceeded 2))")
)
(
  (expr . "(lint-check (quote (def f (lambda (x y) (+ x y z)))) (quote ((max-globals . 0))))")
  (expected . "((globals-exceeded (z)))")
)
(
  (expr . "(lint-check (quote (def f (lambda () (print \"hi\")))) (quote ((max-effects . 0))))")
  (expected . "((effects-exceeded ((print \"hi\"))))")
)
(
  (expr . "(lint-check (quote (def f (lambda (x) x))) (quote ((max-size . 1))))")
  (expected . "((size-exceeded 15))")
)
(
  (expr . "(lint-check (quote (def f (lambda (x) x))) (quote ((max-size . 15) (max-complexity . 5) (max-nesting . 5) (max-globals . 0) (max-effects . 0))))")
  (expected . "()")
)
