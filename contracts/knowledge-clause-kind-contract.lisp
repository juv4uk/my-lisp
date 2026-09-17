; #218 — Lisp-owned clause classification contract.
; A knowledge clause answers with its domain kind, not with historical t/().
; Structural shape is observed below this boundary; clause-kind is the public
; knowledge-domain answer consumed by knowledge readers.

(knowledge-clause-kind-contract/1
  ((query . is-fact?)
   (input . clause)
   (result-layer . knowledge-domain)
   (result-forms . ((clause-kind fact)
                    (clause-kind rule)))
   (generic-truth-coercion . forbidden))

  ((law . empty-body-is-fact)
   (clause-shape . (head))
   (result . (clause-kind fact)))

  ((law . nonempty-body-is-rule)
   (clause-shape . (head condition ...))
   (result . (clause-kind rule))))
