; #305 — presentation of an explicitly established epistemic `unknown` is a
; Lisp-owned law.  This witness deliberately does not infer `unknown` from
; mere absence of proof: `reason-observe` honesty is owned separately by
; tests/fixtures/reason-observe-honesty-v1.lisp and returns Canon 0 when
; neither side has evidence.
;
; Here the richer status is positively constructed with `make-unknown`; the
; presentation layer must keep that status visible rather than collapse it.
; Rust/shell observers see only the named pass envelope.

(load "lib/result-status.lisp")
(load "lib/narrate.lisp")

(def narrate-outcome-authority-check
  (lambda ()
    (let ((actual
            (narrate-outcome
              (make-unknown (quote (parent bob alice)))))
          (expected
            (quote
              (unknown because no-proof-found-for (parent bob alice)))))
      (cond
        ((equal? actual expected) (structural-relation same)
         (quote (narrate-outcome-authority-witness (status pass))))
        ((equal? actual expected) (structural-relation distinct)
         (list
           (quote narrate-outcome-authority-witness)
           (quote (status fail))
           (quote (law explicit-unknown-presentation))
           (list (quote expected) expected)
           (list (quote actual) actual)))))))

(narrate-outcome-authority-check)
