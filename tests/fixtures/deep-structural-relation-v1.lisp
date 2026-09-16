; #218 — deep structural relation witnesses for Lisp-owned equal?.
; equal? reports a structural relation; it does not borrow universal truth.

((expr . "(equal? (quote ()) (quote ()))")
 (expected . "(structural-relation same)")
 (active . t))

((expr . "(equal? (quote radio) (quote radio))")
 (expected . "(structural-relation same)")
 (active . t))

((expr . "(equal? (quote radio) (quote antenna))")
 (expected . "(structural-relation distinct)")
 (active . t))

((expr . "(equal? (quote (radio antenna)) (quote (radio antenna)))")
 (expected . "(structural-relation same)")
 (active . t))

((expr . "(equal? (quote (radio antenna)) (quote (radio signal)))")
 (expected . "(structural-relation distinct)")
 (active . t))

((expr . "(equal? (quote (radio)) (quote radio))")
 (expected . "(structural-relation distinct)")
 (active . t))
