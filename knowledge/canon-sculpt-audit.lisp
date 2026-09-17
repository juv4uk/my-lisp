; #419 PRE-1.0 Canon sculpture audit — integrated evidence snapshot
; Base main: 04084f1003ebd78c946a4b135b8a9febfc7e040e
; Evidence only. This file is NOT semantic authority and does not change language behavior.

(canon-sculpt-audit
  (schema-version 2)
  (base-main 04084f1003ebd78c946a4b135b8a9febfc7e040e)
  (freeze-status pre-1.0-unfrozen)
  (role evidence-only)
  (principle "minimum independent semantic concepts, not minimum lines of code")

  (classification-vocabulary
    (ground-value
     syntax-rule-candidate
     irreducible-candidate
     derivable-candidate
     value-domain
     insufficient-evidence))

  (current-observations
    (canon-runtime-kinds
      (CANON_EMPTY_LIST ground-value)
      (PRIM_QUOTE special-form)
      (PRIM_ATOM value-primitive)
      (PRIM_EQ value-primitive)
      (PRIM_CONS value-primitive)
      (PRIM_CAR value-primitive)
      (PRIM_CDR value-primitive)
      (PRIM_COND special-form))
    (surface-registry
      (quote 0001)
      (atom 0002)
      (eq 0003)
      (cons 0004)
      (car 0005)
      (cdr 0006)
      (cond 0007)
      (if absent))
    (cond-current-canonical-path
      (clause-shape (query expected-result expression))
      (selection exact-value-equality)
      (expected-result quoted-data)
      (evaluation only-selected-consequent)
      (truth-coercion none))
    (cond-migration-path
      (clause-shape (test expression))
      (status compatibility-only)
      (note "temporary truthiness bridge is explicitly not canonical semantics"))
    (library-dependency
      (and derives-through-cond)
      (or derives-through-cond)
      (assoc derives-through-cond)))

  (candidate
    (identity CANON_EMPTY_LIST)
    (category ground-value)
    (classification ground-value)
    (observable-laws
      (ground-object)
      (proper-list-terminator)
      (unspecialized-no-answer-where-explicitly-contracted))
    (falsification-question
      "Can every required role of Canon 0 be represented without introducing another equally fundamental ground identity?"))

  (candidate
    (identity PRIM_QUOTE)
    (category evaluation-control)
    (classification syntax-rule-candidate)
    (observable-laws
      (return-source-form-without-ordinary-evaluation)
      (syntax-only-not-callable))
    (experiment
      (hypothesis ordinary-eager-function-derivation)
      (fixture evidence/canon-sculpt/quote-ordinary-function-red.lisp)
      (predicted-result red)
      (meaning "ordinary eager function application cannot explain suppression of argument evaluation"))
    (remaining-question
      "Is quote irreducible, or derivable from a smaller explicitly declared evaluation-control mechanism that does not already contain quote-equivalent power?"))

  (candidate
    (identity PRIM_COND)
    (category evaluation-control)
    (classification syntax-rule-candidate)
    (observable-laws
      (ordered-clause-selection)
      (query-evaluated-before-match)
      (expected-result-is-data)
      (evaluate-selected-consequent-only)
      (no-value-to-bool-on-canonical-path))
    (negative-evidence
      (if-semantic-identity absent)
      (ordinary-library-and-or-depend-on-cond)
      (machine-lowering-is-not-semantic-derivation))
    (remaining-question
      "Can canonical three-part cond be macro-derived from a smaller declared selective-evaluation mechanism without merely renaming conditional control?"))

  (candidate
    (identity PRIM_ATOM)
    (category structural-observation)
    (classification insufficient-evidence)
    (observable-laws
      ((atom ()) (structural-kind empty-list))
      ((atom pair) (structural-kind pair))
      ((atom non-pair-non-empty) (structural-kind atom)))
    (falsification-question
      "Can structural-kind be derived without atom, an alias/registry round-trip, host representation tags, or a stronger hidden structural classifier?"))

  (candidate
    (identity PRIM_EQ)
    (category identity-observation)
    (classification insufficient-evidence)
    (observable-laws (returns-identity-relation-not-generic-truth))
    (falsification-question
      "Can identity-relation be derived without eq, pointer/host equality, alias round-trip, or a stronger hidden equality primitive?"))

  (candidate
    (identity PRIM_CONS)
    (category structure-construction)
    (classification irreducible-candidate)
    (falsification-question
      "Can pair construction be derived without already possessing pair construction or a stronger aggregate constructor?"))

  (candidate
    (identity PRIM_CAR)
    (category structure-observation)
    (classification insufficient-evidence)
    (falsification-question
      "Can first-component observation be derived without pair destructuring hidden in another operation or substrate load?"))

  (candidate
    (identity PRIM_CDR)
    (category structure-observation)
    (classification insufficient-evidence)
    (falsification-question
      "Can second-component observation be derived without pair destructuring hidden in another operation or substrate load?"))

  (circularity-traps
    direct-self-call
    surface-alias-self-call
    registry-roundtrip-self-call
    host-representation-oracle
    stronger-hidden-primitive
    backend-only-proof)

  (witness-navigation
    (PRIM_ATOM tests/fixtures/structural-observation-v1.lisp)
    (PRIM_EQ tests/fixtures/structural-observation-v1.lisp)
    (PRIM_CONS tests/fixtures/conformance.lisp)
    (PRIM_CAR tests/fixtures/conformance.lisp)
    (PRIM_CDR tests/fixtures/conformance.lisp)
    (PRIM_COND tests/fixtures/conformance.lisp)
    (PRIM_QUOTE evidence/canon-sculpt/quote-ordinary-function-red.lisp))

  (backend-discipline
    (law "lowering proves an implementation path, not semantic derivability")
    (native execution-consumer)
    (meta independent-lisp-evaluator-consumer)
    (cml historical-candidate-needs-current-main-replay)
    (x86-machine partial-mechanism-evidence))

  (coordination
    (merged-neighbor 392)
    (active-neighbors
      421
      422 418 410 406 396
      427
      328
      402 403
      409 424 426 428 429
      317)
    (rule "do not edit their owned production/test routing files from #419"))

  (deletion-gate
    independent-lisp-owned-law-preservation
    explicit-lower-concept-closure
    negative-circularity-proof
    relevant-cross-backend-parity
    owner-review-separate-change
    zero-new-host-semantic-authority)

  (audit-law
    (name no-historical-privilege)
    (statement "Earlier primitive status is evidence to inspect, not proof of irreducibility."))
  (audit-law
    (name uncertainty-wins)
    (statement "If derivability or circularity cannot be demonstrated mechanically, retain insufficient-evidence."))
  (audit-law
    (name preserve-before-delete)
    (statement "No semantic identity is removed in this audit branch.")))
