; #219/#244 — Lisp-owned reasoning honesty contract.
; Pure data: absence of proof is not a proof of negation.

(reasoning-honesty-contract/1
  ((law . no-proof-is-not-negation)
   (observed . no-proof)
   (forbidden-specialization . proved-not)
   (unspecialized-result . ()))

  ((law . explicit-negation-requires-evidence)
   (result . proved-not)
   (requires . explicit-negative-evidence))

  ((law . canon-zero-is-not-refutation)
   (left . ())
   (not-equal . (proved-not refuted false 0/1))))
)
