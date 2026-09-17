; #218 — executable target for PRIM_ATOM / PRIM_EQ domain-owned results.
; Expected outcomes are Lisp-owned semantic data.
;
; #217 now provides a canonical explicit-result dispatch path, so structural
; domain-result rows are active again. Historical two-part cond remains only a
; migration compatibility path while bootstrap/library callers are converted.

((expr . "(atom (quote ()))")
 (expected . "(structural-kind empty-list)")
 (active . t)
 (identity . "0002")
 (case . canon-zero))

((expr . "(atom (quote radio))")
 (expected . "(structural-kind atom)")
 (active . t)
 (identity . "0002")
 (case . non-pair-atom))

((expr . "(atom (quote (radio antenna)))")
 (expected . "(structural-kind pair)")
 (active . t)
 (identity . "0002")
 (case . pair))

((expr . "(eq (quote radio) (quote radio))")
 (expected . "(identity-relation same)")
 (active . t)
 (identity . "0003")
 (case . same-atom))

((expr . "(eq (quote radio) (quote antenna))")
 (expected . "(identity-relation distinct)")
 (active . t)
 (identity . "0003")
 (case . distinct-atoms))

((expr . "(eq (quote (radio)) (quote (radio)))")
 (error . "Type")
 (active . t)
 (identity . "0003")
 (case . outside-atom-domain))

; Vector values are atoms at the PRIM_EQ boundary (only Pair is outside the
; atom domain). Their recursive Value equality therefore belongs to the same
; Lisp-owned identity-relation contract, not to Rust-authored t/() assertions.
((expr . "(eq (vector 1 2 3) (vector 1 2 3))")
 (expected . "(identity-relation same)")
 (active . t)
 (identity . "0003")
 (case . vector-same-structure))

((expr . "(def v (vector 1 2)) (eq v v)")
 (expected . "(identity-relation same)")
 (active . t)
 (identity . "0003")
 (case . vector-same-object))

((expr . "(eq (vector 1 2) (vector 1 9))")
 (expected . "(identity-relation distinct)")
 (active . t)
 (identity . "0003")
 (case . vector-distinct-element))

((expr . "(eq (vector 1 2) (vector 1))")
 (expected . "(identity-relation distinct)")
 (active . t)
 (identity . "0003")
 (case . vector-distinct-length))

((expr . "(eq (vector) (vector))")
 (expected . "(identity-relation same)")
 (active . t)
 (identity . "0003")
 (case . empty-vectors-same))

((expr . "(eq (vector (list 1 2)) (vector (list 1 2)))")
 (expected . "(identity-relation same)")
 (active . t)
 (identity . "0003")
 (case . vector-nested-structure-same))
