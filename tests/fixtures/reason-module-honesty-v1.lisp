; #219 — missing module is an established blocking fact, not epistemic unknown.
; `module-known?` exhausts the concrete append-only knowledge journal. If no
; event for the requested module exists, reasoning never starts: the known
; precondition is absent, so the honest richer result is `blocked`.

((expr . "(reason-in-observe (quote ghost) (quote (planet pluto)))")
 (expected . "(blocked (module-not-found ghost))")
 (active . t)
 (law . missing-module-is-blocked))
