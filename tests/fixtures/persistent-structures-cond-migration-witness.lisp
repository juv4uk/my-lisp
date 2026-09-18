; P0 #613 — Lisp-owned source-shape witness for the persistent-structures
; migration away from historical two-part COND on exact-Q numeric decisions.
;
; Semantic meaning remains in lib/persistent-map.lisp / persistent-vector.lisp
; and their existing behavioral witnesses. This program owns only the migration
; invariant: a clause whose query is < > <= >= = must not have the historical
; two-part shape in these two libraries.

(def persistent-q-comparators
  (quote (< > <= >= =)))

(def persistent-q-member-state
  (lambda (value values)
    (cond
      ((atom values) (structural-kind empty-list)
       (quote (class-membership exact-q-comparator nonmember)))
      ((atom values) (structural-kind atom)
       (quote (class-membership exact-q-comparator nonmember)))
      ((atom values) (structural-kind pair)
       (cond
         ((equal? value (car values)) (structural-relation same)
          (quote (class-membership exact-q-comparator member)))
         ((equal? value (car values)) (structural-relation distinct)
          (persistent-q-member-state value (cdr values))))))))

(def persistent-two-item-state
  (lambda (value)
    (cond
      ((atom value) (structural-kind empty-list) (quote other-shape))
      ((atom value) (structural-kind atom) (quote other-shape))
      ((atom value) (structural-kind pair)
       (let ((rest1 (cdr value)))
         (cond
           ((atom rest1) (structural-kind empty-list) (quote other-shape))
           ((atom rest1) (structural-kind atom) (quote other-shape))
           ((atom rest1) (structural-kind pair)
            (let ((rest2 (cdr rest1)))
              (cond
                ((atom rest2) (structural-kind empty-list) (quote two-item))
                ((atom rest2) (structural-kind atom) (quote other-shape))
                ((atom rest2) (structural-kind pair) (quote other-shape)))))))))))

(def persistent-numeric-query-state
  (lambda (query)
    (cond
      ((atom query) (structural-kind empty-list)
       (quote (class-membership exact-q-comparator nonmember)))
      ((atom query) (structural-kind atom)
       (quote (class-membership exact-q-comparator nonmember)))
      ((atom query) (structural-kind pair)
       (persistent-q-member-state (car query) persistent-q-comparators)))))

(def persistent-clause-exact-q-two-part-count
  (lambda (clause)
    (cond
      ((equal? (persistent-two-item-state clause) (quote two-item))
       (structural-relation same)
       (cond
         ((persistent-numeric-query-state (car clause))
          (class-membership exact-q-comparator member)
          1)
         ((persistent-numeric-query-state (car clause))
          (class-membership exact-q-comparator nonmember)
          0)))
      ((equal? (persistent-two-item-state clause) (quote two-item))
       (structural-relation distinct)
       0))))

(def persistent-cond-clauses-exact-q-two-part-count
  (lambda (clauses)
    (cond
      ((atom clauses) (structural-kind empty-list) 0)
      ((atom clauses) (structural-kind atom) 0)
      ((atom clauses) (structural-kind pair)
       (+ (persistent-clause-exact-q-two-part-count (car clauses))
          (persistent-cond-clauses-exact-q-two-part-count (cdr clauses)))))))

(def persistent-form-local-cond-count
  (lambda (form)
    (cond
      ((atom form) (structural-kind empty-list) 0)
      ((atom form) (structural-kind atom) 0)
      ((atom form) (structural-kind pair)
       (cond
         ((equal? (car form) (quote cond)) (structural-relation same)
          (persistent-cond-clauses-exact-q-two-part-count (cdr form)))
         ((equal? (car form) (quote cond)) (structural-relation distinct)
          0))))))

(def persistent-form-exact-q-two-part-count
  (lambda (form)
    (cond
      ((atom form) (structural-kind empty-list) 0)
      ((atom form) (structural-kind atom) 0)
      ((atom form) (structural-kind pair)
       (cond
         ((equal? (car form) (quote quote)) (structural-relation same) 0)
         ((equal? (car form) (quote quote)) (structural-relation distinct)
          (+ (persistent-form-local-cond-count form)
             (persistent-forms-exact-q-two-part-count form))))))))

(def persistent-forms-exact-q-two-part-count
  (lambda (forms)
    (cond
      ((atom forms) (structural-kind empty-list) 0)
      ((atom forms) (structural-kind atom) 0)
      ((atom forms) (structural-kind pair)
       (+ (persistent-form-exact-q-two-part-count (car forms))
          (persistent-forms-exact-q-two-part-count (cdr forms)))))))

(def persistent-map-two-part-count
  (persistent-forms-exact-q-two-part-count
    (read-all (read-file "lib/persistent-map.lisp"))))

(def persistent-vector-two-part-count
  (persistent-forms-exact-q-two-part-count
    (read-all (read-file "lib/persistent-vector.lisp"))))

(def persistent-cond-migration-fail
  (lambda ()
    (let ((shown
            (print
              (list
                (quote persistent-structures-cond-migration-witness)
                (list (quote status) (quote fail))
                (list (quote persistent-map-two-part)
                      persistent-map-two-part-count)
                (list (quote persistent-vector-two-part)
                      persistent-vector-two-part-count)))))
      ; Deliberately unbound: a failed migration witness must make CI non-zero.
      (persistent-exact-q-two-part-cond-remains shown))))

(cond
  ((= persistent-map-two-part-count 0) 1
   (cond
     ((= persistent-vector-two-part-count 0) 1
      (print
        (quote
          (persistent-structures-cond-migration-witness
            (status pass)))))
     ((= persistent-vector-two-part-count 0) 0
      (persistent-cond-migration-fail))))
  ((= persistent-map-two-part-count 0) 0
   (persistent-cond-migration-fail)))
