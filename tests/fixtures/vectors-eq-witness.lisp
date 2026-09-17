; tests/fixtures/vectors-eq-witness.lisp — Lisp-owned expected outcomes for
; vector `eq` structural comparison (#218's identity-relation contract).
; Rust (crates/my-lisp/tests/vectors.rs) transports the actual outcome and
; asks tests/fixtures/witness-runner.lisp's generic witness-verdict for the
; verdict; it never authors the expected value itself.

((expr . "(eq (vector 1 2 3) (vector 1 2 3))") (expected . "(identity-relation same)"))
((expr . "(def v (vector 1 2)) (eq v v)") (expected . "(identity-relation same)"))
((expr . "(eq (vector 1 2) (vector 1 9))") (expected . "(identity-relation distinct)"))
((expr . "(eq (vector 1 2) (vector 1))") (expected . "(identity-relation distinct)"))
((expr . "(eq (vector) (vector))") (expected . "(identity-relation same)"))
((expr . "(eq (vector (list 1 2)) (vector (list 1 2)))") (expected . "(identity-relation same)"))
