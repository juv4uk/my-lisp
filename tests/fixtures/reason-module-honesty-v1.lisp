; #219 — missing module is an established blocking fact, not epistemic unknown.
; `module-known?` exhausts the concrete append-only knowledge journal. If no
; event for the requested module exists, reasoning never starts: the known
; precondition is absent, so the honest richer result is `blocked`.
;
; #369 also preserves the stronger invalid-module result when the module name
; is not a symbol. That classification must survive the 1023 result-domain
; migration without depending on the historical t/() predicate sentinel.

((expr . "(list (reason-in-observe (quote ghost) (quote (planet pluto))) (reason-in-observe 42 (quote (planet pluto))))")
 (expected . "((blocked (module-not-found ghost)) (invalid invalid-module 42))")
 (active . t)
 (law . missing-module-and-invalid-module-honesty))
