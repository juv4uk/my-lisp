; #150 — executable one-way machine authority guard.
; The normal input contains admitted semantic-to-machine edges only.
; CI replaces that input with a committed reverse-edge fixture and requires
; this program to fail with machine-authority-boundary-violation.

(def second (lambda (x) (car (cdr x))))
(def third (lambda (x) (car (cdr (cdr x)))))

(def authority-edges
  (read-all (read-file "tests/machine-authority-edges.lisp")))

(def allowed-machine-edge?
  (lambda (row)
    (cond
      ((eq (car row) (quote authority-edge))
       (cond
         ((eq (second row) (quote semantic))
          (cond
            ((eq (third row) (quote machine)) t)
            (t ())))
         (t ())))
      (t ()))))

; Deliberately call an unbound diagnostic symbol.  The language error itself
; therefore carries the authority-boundary diagnostic even when stdout is
; buffered; CI observes the non-zero exit plus this exact diagnostic name.
(def fail-machine-authority
  (lambda (row)
    (machine-authority-boundary-violation row)))

(def check-machine-edges
  (lambda (rows)
    (cond
      ((atom rows) t)
      ((allowed-machine-edge? (car rows))
       (check-machine-edges (cdr rows)))
      (t (fail-machine-authority (car rows))))))

(check-machine-edges authority-edges)
