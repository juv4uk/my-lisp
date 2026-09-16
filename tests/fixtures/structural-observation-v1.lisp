; #218 — executable target for PRIM_ATOM / PRIM_EQ domain-owned results.
; Expected outcomes are Lisp-owned semantic data.
;
; #217 now provides a canonical explicit-result dispatch path, so the five
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
