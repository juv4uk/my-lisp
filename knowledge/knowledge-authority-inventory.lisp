; #383 — machine-readable classification of knowledge artifacts.
; Directory placement never creates semantic authority.
; RED slice: only the registry classifies itself; live coverage must reject
; the remaining observed knowledge artifacts until the inventory is completed.

(about
  (schema knowledge-authority-inventory/1)
  (issue 383)
  (scope knowledge-artifact-governance)
  (authority-rule provenance-and-scope-not-directory))

(artifact
  (path "knowledge/knowledge-authority-inventory.lisp")
  (class operational-reference)
  (scope knowledge-artifact-governance)
  (authority-source (issue 383))
  (lifecycle active)
  (consumers ("scripts/check-knowledge-authority.lisp")))
