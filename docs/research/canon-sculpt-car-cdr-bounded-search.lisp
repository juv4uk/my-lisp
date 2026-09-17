; #432 bounded structural-basis search
; Research-only data. NOT semantic authority. NOT a proof of global irreducibility.
; Base: main@23cfdc141ef1e7ab4275d27c536a99d1018f3292
;
; Purpose:
;   Falsify a concrete family of CAR/CDR reductions without touching Canon/runtime.
;   The search asks whether a unary expression over a deliberately restricted
;   lower basis can reproduce CAR or CDR on finite separating probe families.
;
; Important distinction:
;   the search mechanism may inspect generated terms as research data;
;   generated object-language candidates themselves may not contain CAR/CDR,
;   peer surfaces, semantic-ID round trips, host pair destructuring, or backend loads.

(canon-sculpt-bounded-search
  (schema-version 1)
  (issue 432)
  (base-main "23cfdc141ef1e7ab4275d27c536a99d1018f3292")
  (role research-only)
  (classification insufficient-evidence)

  (object-language-basis
    (variable x)
    (constants
      () a b c h z
      (structural-kind pair)
      (structural-kind atom)
      (structural-kind empty-list)
      (identity-relation same)
      (identity-relation distinct))
    (constructors
      atom
      eq
      cons
      exact-result-selector)
    (exact-result-selector
      (meaning "canonical three-part COND power abstracted as exact query==datum selection")
      (truth-coercion forbidden)
      (selected-branch-only t))
    (forbidden
      car cdr
      peer-surface-car peer-surface-cdr
      semantic-registry-roundtrip
      host-pair-destructuring
      representation-tag-oracle
      backend-load-as-semantic-primitive))

  (search-method
    (kind extensional-behavior-closure)
    (rounds 2)
    (deduplication "candidate terms with identical observations on all probes collapse to one behavior vector")
    (error-model "eq applied to a pair yields named Type failure; failures propagate through eager arguments; unselected selector branch is not evaluated")
    (round-0-behaviors 12)
    (round-1-new-behaviors 145)
    (round-1-total-behaviors 157)
    (round-2-new-behaviors 24192)
    (round-2-total-behaviors 24349)
    (round-2-varying-behaviors 6913))

  (car-separation
    (inputs
      ((a . z) (b . z) (c . z)))
    (target-observation (a b c))
    (property "same tail, three distinct heads")
    (result target-not-found-through-round-2)
    (interpretation
      "Within this finite grammar/depth, no candidate without pair projection reproduced CAR on the separating family."))

  (cdr-separation
    (inputs
      ((h . a) (h . b) (h . c)))
    (target-observation (a b c))
    (property "same head, three distinct tails")
    (result target-not-found-through-round-2)
    (interpretation
      "Within this finite grammar/depth, no candidate without pair projection reproduced CDR on the separating family."))

  (why-these-probes-separate
    (car
      "ATOM sees pair/pair/pair; EQ cannot inspect pair interiors; CONS can only wrap available values; exact selection can branch only on already observable distinctions. The varying head is therefore the hidden coordinate the candidate must somehow recover.")
    (cdr
      "The dual family holds the head fixed and varies only the tail, so the missing observation is the second coordinate."))

  (non-claims
    global-irreducibility-not-proven
    all-possible-lower-bases-not-enumerated
    unbounded-program-space-not-searched
    implementation-mechanism-not-semantic-proof)

  (next-falsification
    (increase-bound "extend closure beyond round 2 with a repository-executable verifier")
    (candidate-basis-test "if a proposed weaker pair eliminator appears, add it explicitly and rerun rather than silently strengthening the basis")
    (positive-stop "if any candidate reaches the target observation, inspect it for hidden projection/circularity before calling it a derivation")))
