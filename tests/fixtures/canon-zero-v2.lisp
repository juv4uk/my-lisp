; #215 EMPTY-LIST-STRUCTURE-1 — Lisp-owned Canon 0 transition witnesses.
; Expected semantic outcomes live here. Rust may only transport actual outcomes
; to tests/fixtures/witness-runner.lisp for comparison.
;
; ACTIVE rows prove the early structural half of #215 now:
;   * () remains ordinary empty-list data and a proper-list terminator;
;   * structural empty and numeric zero remain observably distinct values.
;
; BLOCKED rows preserve the already-proven RED target for the final half:
;   * producing () must not be consumed as binary FALSE by control flow.
; They are intentionally not active until #218 gives structural observations an
; explicit result algebra and #217 gives control an explicit decision protocol.
; We do not add a hidden bootstrap/legacy-cond mode because semantics must not
; depend on loader context.
;
; Rich-reasoning zero-answer coverage belongs to #219; #215 remains open until
; the cross-domain witness is present and the blocked control rows can activate.

((expr . "(quote ())")
 (expected . "()")
 (owner . canon-zero)
 (active . t)
 (note . "Canon 0 materializes as the empty list, not a truth sentinel"))

((expr . "(cdr (quote (ground)))")
 (expected . "()")
 (owner . canon-zero)
 (active . t)
 (note . "proper-list termination still produces the same structural empty list"))

((expr . "(cons (quote ()) (cons 0/1 (quote ())))")
 (expected . "(() 0)")
 (owner . canon-zero)
 (active . t)
 (note . "empty-list data and exact numeric zero remain distinct list elements"))

((expr . "(cond (() (quote forbidden)) (t (quote legacy-false)))")
 (error . "Type")
 (owner . canon-zero)
 (blocked-by . control-logic-217)
 (depends-on . structural-observation-218)
 (note . "RED proven in PR #235: literal empty list must eventually be rejected as a decision, but activating this before #218/#217 breaks Lisp bootstrap"))

((expr . "(cond ((cdr (quote (ground))) (quote forbidden)) (t (quote legacy-false)))")
 (error . "Type")
 (owner . canon-zero)
 (blocked-by . control-logic-217)
 (depends-on . structural-observation-218)
 (note . "RED proven in PR #235: an operation producing () must not become FALSE; retained as future executable target"))
