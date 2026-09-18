; Experiment: unfold the semantic table from the empty list.
;
; This file deliberately does NOT define a Canon and does NOT assign meaning
; to registry spellings. It asks a smaller question:
;
;   can one ordinary Lisp value represent the whole table, with () as the
;   only terminal ground and pairs as the only recursive structure?
;
; The host must not know "en", "uk", "car", "cond", or "registry" to hold this
; value. Those are leaves in the same tree.
;
; Phase 0: the ground.
(def table-ground (quote ()))
;
; Phase 1: one cell.  A non-empty list is a pair whose tail eventually reaches
; table-ground.  Nothing registry-specific is introduced here.
(def table-cell
  (lambda (head tail)
    (cons head tail)))
;
; Phase 2: one row is only a list assembled from cells.
; We intentionally use a variadic tail rather than a Row/Column host type.
(def table-row
  (lambda fields fields))
;
; Phase 3: the table is only a list of rows.  Again, no Table host type.
(def table
  (lambda rows rows))
;
; Smallest witness.  Every branch terminates in ().
(def empty-list-unfold-witness
  (table
    (table-row
      (quote 00000000)
      (table-row (quote en)  table-ground)
      (table-row (quote uk)  table-ground)
      (table-row (quote ukr) table-ground)
      (table-row (quote sa)  table-ground)
      (table-row (quote sym) table-ground))))
;
; Structural observer using the post-revolution explicit ATOM result.
; COND does not consume truthiness here: it matches the observed result
; against an explicit expected Lisp value.
(def table-shape
  (lambda (value)
    (cond
      ((atom value) (structural-kind empty-list)
       (quote ()))
      ((atom value) (structural-kind pair)
       (cons (table-shape (car value))
             (table-shape (cdr value))))
      ((atom value) (structural-kind atom)
       value))))
;
; If this evaluates to the same tree as empty-list-unfold-witness, Lisp can
; traverse the witness without any registry-specific reader/row/column logic.
(def empty-list-unfold-result
  (table-shape empty-list-unfold-witness))

;
; Phase 4: the first self-identification probe.
; No host code is allowed to state that () means 00000000.  The relation is
; present only in the Lisp witness: the first field is the opaque identity and
; every remaining field in this experimental row points to table-ground.
;
; For the one-row experiment, ask the structure for the identity associated
; with the ground value.  This deliberately knows only list structure, not the
; words en/uk/ukr/sa/sym and not the meaning of 00000000.
(def ground-row?
  (lambda (row)
    (cond
      ((atom (cdr row)) (structural-kind empty-list)
       (eq (quote ()) (quote ())))
      ((atom (car (cdr (car (cdr row))))) (structural-kind empty-list)
       (ground-row? (cons (car row) (cdr (cdr row)))))
      ((atom (car (cdr (car (cdr row))))) (structural-kind atom)
       (eq (quote ()) (quote not-empty)))
      ((atom (car (cdr (car (cdr row))))) (structural-kind pair)
       (eq (quote ()) (quote not-empty))))))

(def identify-ground
  (lambda (rows)
    (cond
      ((atom rows) (structural-kind empty-list)
       (quote ()))
      ((ground-row? (car rows)) (eq (quote ()) (quote ()))
       (car (car rows)))
      ((ground-row? (car rows)) (eq (quote ()) (quote not-empty))
       (identify-ground (cdr rows))))))

(def empty-list-identity
  (identify-ground empty-list-unfold-witness))
