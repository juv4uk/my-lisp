; #305 — numeric-buffer identity observations belong to Lisp-owned semantic
; evidence, not to historical Rust assertions that hard-code t/().
;
; Preserve the three buffer-specific laws currently exposed by EQ before the
; stale host-side truth oracles are retired:
;   * equal i32 buffer values -> (identity-relation same)
;   * i32 vs f32 buffer values -> (identity-relation distinct)
;   * f32 -0.0 vs +0.0 -> (identity-relation distinct)
;
; Deliberately NOT preserved here: `(eq (numeric-buffer? ...) t)`.  Identity
; 1026 now has a ratified class-membership result algebra under #218, so the
; old truth-sentinel/Bool representation check is migration debt, not a law.

(def numeric-buffer-identity-expect
  (lambda (law actual expected)
    (cond
      ((equal? actual expected) (structural-relation same)
       (quote ()))
      ((equal? actual expected) (structural-relation distinct)
       (list
         (quote numeric-buffer-identity-witness)
         (quote (status fail))
         (list (quote law) law)
         (list (quote expected) expected)
         (list (quote actual) actual))))))

(def numeric-buffer-identity-check
  (lambda ()
    (let ((same-i32
            (numeric-buffer-identity-expect
              (quote equal-i32-values)
              (eq (i32-buffer 1 2) (i32-buffer 1 2))
              (quote (identity-relation same))))
          (cross-type
            (numeric-buffer-identity-expect
              (quote i32-vs-f32-distinct)
              (eq (i32-buffer 1) (f32-buffer 1))
              (quote (identity-relation distinct))))
          (signed-zero
            (numeric-buffer-identity-expect
              (quote f32-signed-zero-distinct)
              (eq #f32(-0.0) #f32(0.0))
              (quote (identity-relation distinct)))))
      (cond
        ((equal? same-i32 (quote ())) (structural-relation distinct)
         same-i32)
        ((equal? cross-type (quote ())) (structural-relation distinct)
         cross-type)
        ((equal? signed-zero (quote ())) (structural-relation distinct)
         signed-zero)
        ((equal? (quote pass) (quote pass)) (structural-relation same)
         (quote (numeric-buffer-identity-witness (status pass))))))))

(numeric-buffer-identity-check)
