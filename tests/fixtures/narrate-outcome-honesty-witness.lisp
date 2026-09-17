; #305/#219 — preserve the reasoning/narration boundary before retiring a
; stale Rust oracle. Neither-side-not-proved is Canon 0 unless a named
; completeness/search contract establishes a richer epistemic status.
; Presentation must not manufacture `unknown` from that unspecialized result.
;
; Rust/shell may observe only the final named pass/fail envelope.

(load "lib/unify.lisp")
(load "lib/reason.lisp")
(load "lib/result-status.lisp")
(load "lib/narrate.lisp")

(def narrate-honesty-failure
  (lambda (case actual expected)
    (list
      (quote narrate-outcome-honesty-witness)
      (quote (status fail))
      (list (quote case) case)
      (list (quote actual) actual)
      (list (quote expected) expected))))

(let* ((rules (quote (((parent alice bob)))))
       (goal (quote (parent bob alice)))
       (outcome (reason-observe goal rules))
       (narration (narrate-outcome outcome)))
  (cond
    ((equal? outcome (quote ())) (structural-relation same)
     (cond
       ((equal? narration (quote (invalid outcome-shape ()))) (structural-relation same)
        (quote (narrate-outcome-honesty-witness (status pass))))
       ((equal? narration (quote (invalid outcome-shape ()))) (structural-relation distinct)
        (narrate-honesty-failure
          (quote presentation-must-not-invent-unknown)
          narration
          (quote (invalid outcome-shape ()))))))
    ((equal? outcome (quote ())) (structural-relation distinct)
     (narrate-honesty-failure
       (quote no-evidence-remains-unspecialized)
       outcome
       (quote ())))))
