(def build-map
  (lambda (keys map)
    (cond
      ((atom keys) map)
      (t (build-map (cdr keys) (map-insert (car keys) (car keys) map))))))

(def map-keys-sorted?
  (lambda (l)
    (cond
      ((atom l) (structural-kind empty-list) t)
      ((atom (cdr l)) (structural-kind empty-list) t)
      ((identity-relation same) (identity-relation same)
       (let ((order (string-order (car (car (cdr l))) (car (car l)))))
         (cond
           ((eq order (quote (text-order before)))
            (identity-relation same)
            (quote ()))
           ((eq order (quote (text-order same)))
            (identity-relation same)
            (map-keys-sorted? (cdr l)))
           ((eq order (quote (text-order after)))
            (identity-relation same)
            (quote ()))))))))

(def fib
  (lambda (n)
    (cond
      ((< n 2) n)
      (t (+ (fib (+ n -1)) (fib (+ n -2)))))))

(def build-world
  (lambda (events w)
    (cond
      ((atom events) w)
      (t (build-world (cdr events) (world-tell w "mod" (car events)))))))

(def tell-all-events
  (lambda (events)
    (cond
      ((atom events) t)
      (t (let ((_ (defmodule "mod" (list (car events)))))
           (tell-all-events (cdr events)))))))
