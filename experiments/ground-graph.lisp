; Experimental ground graph.
; No registry rows, columns, surface tags, or Canon live here.
; Every machine identity/relation marker is an opaque BitPattern8.
;
; Apparatus still introduced by the current Lisp runtime:
; list/pair traversal, atom/eq/cond, and the reader.
; The experiment asks whether one generic graph mechanism can express both
; equivalence and transition without knowing the meaning of endpoint values.

; E-class-inspired peer set: two representations coexist without destructive
; conversion. This is intentionally tiny; it borrows the e-class idea, not a
; full equality-saturation engine.
(def ground-eclass
  (list (quote ()) 00000000))

(def two-member-eclass-peer
  (lambda (class value)
    (let ((left (car class)))
      (let ((right (second class)))
        (cond
          ((eq value left) (identity-relation same) right)
          ((eq value left) (identity-relation distinct)
           (cond
             ((eq value right) (identity-relation same) left)
             ((eq value right) (identity-relation distinct) (quote ())))))))))

; Opaque relation identities. Their meanings are not encoded in their bits.
(def ground-equivalence-relation 00000010)
(def ground-transition-relation 00000011)

; Generic edge shape: (endpoint relation endpoint).
; An edge is traversable in either direction by the same mechanism.
(def ground-graph
  (list
    (list (quote ()) ground-equivalence-relation 00000000)
    (list 00000000 ground-transition-relation 00000001)))

(def graph-edge-neighbor
  (lambda (edge relation value)
    (let ((left (car edge)))
      (let ((edge-relation (second edge)))
        (let ((right (third edge)))
          (cond
            ((eq edge-relation relation) (identity-relation same)
             (cond
               ((eq left value) (identity-relation same) right)
               ((eq left value) (identity-relation distinct)
                (cond
                  ((eq right value) (identity-relation same) left)
                  ((eq right value) (identity-relation distinct) (quote ()))))))
            ((eq edge-relation relation) (identity-relation distinct)
             (quote ()))))))))

(def graph-neighbor
  (lambda (graph relation value)
    (cond
      ((atom graph) (structural-kind empty-list) (quote ()))
      ((atom graph) (structural-kind pair)
       (let ((candidate (graph-edge-neighbor (car graph) relation value)))
         (cond
           ((atom candidate) (structural-kind empty-list)
            (graph-neighbor (cdr graph) relation value))
           ((atom candidate) (structural-kind atom) candidate)
           ((atom candidate) (structural-kind pair) candidate))))
      ((atom graph) (structural-kind atom) (quote ())))))

(def ground-graph-witness
  (lambda ()
    (list
      (list
        (quote eclass-forward)
        (two-member-eclass-peer ground-eclass (quote ())))
      (list
        (quote eclass-backward)
        (two-member-eclass-peer ground-eclass 00000000))
      (list
        (quote graph-equivalence-forward)
        (graph-neighbor ground-graph ground-equivalence-relation (quote ())))
      (list
        (quote graph-equivalence-backward)
        (graph-neighbor ground-graph ground-equivalence-relation 00000000))
      (list
        (quote graph-transition-forward)
        (graph-neighbor ground-graph ground-transition-relation 00000000))
      (list
        (quote graph-transition-backward)
        (graph-neighbor ground-graph ground-transition-relation 00000001)))))

(ground-graph-witness)
