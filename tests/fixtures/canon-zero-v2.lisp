; #215 EMPTY-LIST-STRUCTURE-1 — Lisp-owned Canon 0 transition witnesses.
; Expected semantic outcomes live here. Rust may only transport actual outcomes
; to tests/fixtures/witness-runner.lisp for comparison.
;
; This bounded slice proves:
;   * () remains ordinary empty-list data and a proper-list terminator;
;   * structural empty and numeric zero remain observably distinct values;
;   * producing () cannot be consumed as binary FALSE by current control flow.
;
; The complete future cond protocol belongs to #217. Rich-reasoning zero-answer
; coverage belongs to #219; #215 remains open until that cross-domain witness is
; present.

((expr . "(quote ())")
 (expected . "()")
 (owner . canon-zero)
 (note . "Canon 0 materializes as the empty list, not a truth sentinel"))

((expr . "(cdr (quote (ground)))")
 (expected . "()")
 (owner . canon-zero)
 (note . "proper-list termination still produces the same structural empty list"))

((expr . "(cons (quote ()) (cons 0/1 (quote ())))")
 (expected . "(() 0)")
 (owner . canon-zero)
 (note . "empty-list data and exact numeric zero remain distinct list elements"))

((expr . "(cond (() (quote forbidden)) (t (quote legacy-false)))")
 (error . "Type")
 (owner . canon-zero)
 (note . "literal empty list may not be consumed as FALSE; #217 will replace remaining generic truthiness"))

((expr . "(cond ((cdr (quote (ground))) (quote forbidden)) (t (quote legacy-false)))")
 (error . "Type")
 (owner . canon-zero)
 (note . "an operation that produces () may not turn that empty result into FALSE"))
