; #218 — executable target for PRIM_ATOM / PRIM_EQ domain-owned results.
; Expected outcomes are Lisp-owned semantic data.
;
; Five value-result rows were run active in PR #238 and produced the intended
; RED: current runtime returned historical t/() instead of the domain-owned
; records below. They stay committed but blocked until #217 gives `cond` and
; bootstrap code an explicit structural dispatch protocol. No hidden loader or
; legacy-cond mode is allowed.
;
; The final Type row is already compatible with the selected eq input domain and
; remains active now.

((expr . "(atom (quote ()))")
 (expected . "(structural-kind empty-list)")
 (blocked-by . control-logic-217)
 (identity . "0002")
 (case . canon-zero))

((expr . "(atom (quote radio))")
 (expected . "(structural-kind atom)")
 (blocked-by . control-logic-217)
 (identity . "0002")
 (case . non-pair-atom))

((expr . "(atom (quote (radio antenna)))")
 (expected . "(structural-kind pair)")
 (blocked-by . control-logic-217)
 (identity . "0002")
 (case . pair))

((expr . "(eq (quote radio) (quote radio))")
 (expected . "(identity-relation same)")
 (blocked-by . control-logic-217)
 (identity . "0003")
 (case . same-atom))

((expr . "(eq (quote radio) (quote antenna))")
 (expected . "(identity-relation distinct)")
 (blocked-by . control-logic-217)
 (identity . "0003")
 (case . distinct-atoms))

((expr . "(eq (quote (radio)) (quote (radio)))")
 (error . "Type")
 (active . t)
 (identity . "0003")
 (case . outside-atom-domain))
