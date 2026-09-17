; #293/#295 — preserve useful laws retired with legacy Rust truth sentinels.
; Semantic expected relations live in Lisp. Rust may only observe the named
; pass/fail envelope.
;
; These are boundary laws, not a restoration of universal t/() truth:
; 1. reader reconstruction of a printed dotted pair preserves structure;
; 2. an exact-Q comparison result survives a macro expansion/re-evaluation
;    boundary as the same semantic value.

(defmacro preservation-exact-no () (< 2 1))

(def retired-mccarthy-boundary-rows
  (lambda ()
    (list
      (list
        (quote dotted-pair-reader-round-trip)
        (equal? (read "(p . 0)") (cons (quote p) 0))
        (quote (structural-relation same)))
      (list
        (quote macro-boundary-result-invariance)
        (equal? (< 2 1) (preservation-exact-no))
        (quote (structural-relation same))))))

(def retired-mccarthy-boundary-check
  (lambda (rows)
    (cond
      ((atom rows) (structural-kind empty-list)
       (quote (retired-mccarthy-boundary-witness (status pass))))
      ((atom rows) (structural-kind atom)
       (list
         (quote retired-mccarthy-boundary-witness)
         (list (quote status) (quote fail))
         (list (quote case) (quote malformed-row-tail))
         (list (quote actual) rows)))
      ((atom rows) (structural-kind pair)
       (let ((row (car rows)))
         (cond
           ((equal? (second row) (third row)) (structural-relation same)
            (retired-mccarthy-boundary-check (cdr rows)))
           ((equal? (second row) (third row)) (structural-relation distinct)
            (list
              (quote retired-mccarthy-boundary-witness)
              (list (quote status) (quote fail))
              (list (quote case) (car row))
              (list (quote actual) (second row))
              (list (quote expected) (third row))))))))))

(def retired-mccarthy-boundary-witness
  (lambda ()
    (retired-mccarthy-boundary-check (retired-mccarthy-boundary-rows))))
