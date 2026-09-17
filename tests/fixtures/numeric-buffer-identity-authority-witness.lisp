; #370 — typed numeric-buffer identity is a Lisp-owned semantic law.
; This witness preserves only the useful identity relation that was previously
; duplicated by a Rust assertion.  It deliberately does NOT preserve the
; historical numeric-buffer? -> t/() representation: #344 already ratified
; that predicate's future result algebra as explicit class-membership data.
; Rust/shell observers see only the named pass envelope.

(def numeric-buffer-identity-authority-check
  (lambda ()
    (let ((actual
            (eq (i32-buffer 1 2) (i32-buffer 1 2)))
          (expected
            (quote (identity-relation same))))
      (cond
        ((equal? actual expected) (structural-relation same)
         (quote (numeric-buffer-identity-authority-witness (status pass))))
        ((equal? actual expected) (structural-relation distinct)
         (list
           (quote numeric-buffer-identity-authority-witness)
           (quote (status fail))
           (quote (law typed-buffer-identity))
           (list (quote expected) expected)
           (list (quote actual) actual)))))))

(numeric-buffer-identity-authority-check)
