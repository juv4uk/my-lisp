; answer-contract.lisp — Lisp-owned answer/domain authority for #228.
;
; This is deliberately a pure Lisp data document, not an executable library.
; It declares semantic authority without creating public API names or depending
; on the evaluator's historical T/NIL truth machinery merely to exist.
;
; Human spellings remain owned by lib/surface/semantic-registry.lisp.
; Canon 0 is the sole non-ID entry because the empty list itself has no lexical
; semantic-ID spelling.
;
; Governing non-interference rules from #214/#222/#227:
;   * () is the empty list / zero-answer result, never FALSE.
;   * exact-Q binary decisions own only 0/1 and 1/1.
;   * non-binary mathematics remains mathematical result data.
;   * richer reasoning is non-mathematical and cannot redefine mathematics.
;   * arbitrary Value -> bool and host-bool truth authority are forbidden.
;   * lower domains may refine delegated territory; they may not redefine
;     upper domains.
;
; The outer form is:
;   (answer-contract/1 ENTRY ...)
; Consumers read it as data. #228's Lisp-owned witness is the first executable
; consumer. Later semantic slices may add runtime consumers without changing
; this authority boundary.

(answer-contract/1
  ((identity . canon-zero)
   (domain-owner . no-answer-boundary)
   (input-domain . none)
   (result-form . empty-list)
   (no-answer . self)
   (allowed-coercions . ())
   (forbidden-coercions . (false-sentinel binary-collapse generic-value->bool host-bool)))

  ((identity . "0001")
   (domain-owner . structure)
   (input-domain . syntax)
   (result-form . data)
   (no-answer . not-applicable)
   (allowed-coercions . ())
   (forbidden-coercions . (generic-value->bool host-bool)))

  ((identity . "0004")
   (domain-owner . structure)
   (input-domain . (value value))
   (result-form . pair)
   (no-answer . not-applicable)
   (allowed-coercions . ())
   (forbidden-coercions . (generic-value->bool host-bool)))

  ((identity . "0005")
   (domain-owner . structure)
   (input-domain . pair)
   (result-form . value)
   (no-answer . not-applicable)
   (allowed-coercions . ())
   (forbidden-coercions . (generic-value->bool host-bool)))

  ((identity . "0006")
   (domain-owner . structure)
   (input-domain . pair)
   (result-form . value)
   (no-answer . not-applicable)
   (allowed-coercions . ())
   (forbidden-coercions . (generic-value->bool host-bool)))

  ; #218 will ratify the concrete result algebra for structural observations.
  ; #228 only establishes that atom/eq do NOT inherit mathematical binary
  ; semantics or generic truthiness.
  ((identity . "0002")
   (domain-owner . structural-observation)
   (input-domain . value)
   (result-form . structural-observation-result)
   (no-answer . ())
   (allowed-coercions . ())
   (forbidden-coercions . (generic-value->bool host-bool mathematical-binary-collapse)))

  ((identity . "0003")
   (domain-owner . structural-observation)
   (input-domain . (value value))
   (result-form . structural-observation-result)
   (no-answer . ())
   (allowed-coercions . ())
   (forbidden-coercions . (generic-value->bool host-bool mathematical-binary-collapse)))

  ; < is the first exact-Q decision witness. outside-domain=delegate is
  ; essential: a non-Q question is not binary NO. A lower appropriate domain
  ; may answer it, or the whole query may ultimately return ().
  ((identity . "1014")
   (domain-owner . exact-q-decision)
   (input-domain . exact-rational-sequence)
   (result-form . rational-binary-decision)
   (binary-values . ("0/1" "1/1"))
   (outside-domain . delegate)
   (no-answer . ())
   (allowed-coercions . (exact-rational-normalization))
   (forbidden-coercions . (generic-value->bool host-bool truthy-collapse approximate-to-rational)))

  ; + witnesses the separate mathematical-result floor. A mathematical value
  ; does not become a truth status merely because it is non-binary or non-Q.
  ((identity . "0104")
   (domain-owner . mathematical-result)
   (input-domain . mathematical-values)
   (result-form . mathematical-value)
   (no-answer . ())
   (allowed-coercions . (domain-valid-mathematical-promotion))
   (forbidden-coercions . (truth-collapse many-valued-collapse generic-value->bool host-bool)))

  ; reason keeps its current public shape for now: a list of
  ; (substitution proof) answers, or () when no proof/result is produced.
  ; #223/#224/#219 may refine this lower reasoning algebra later without
  ; acquiring authority over structure or mathematics.
  ((identity . "1118")
   (domain-owner . non-mathematical-reasoning)
   (input-domain . (goal rules-or-index))
   (result-form . reasoning-results)
   (no-answer . ())
   (allowed-coercions . (reasoning-domain-refinement))
   (forbidden-coercions . (binary-collapse mathematical-collapse generic-value->bool host-bool)))

  ; cond consumes an explicit answer/result domain; it does not own universal
  ; truth. #217 will ratify the exact dispatch protocol.
  ((identity . "0007")
   (domain-owner . control-consumer)
   (input-domain . explicit-answer-result)
   (result-form . selected-branch-value)
   (no-answer . ())
   (allowed-coercions . (domain-dispatch))
   (forbidden-coercions . (generic-value->bool host-bool truthy-collapse))
   (generic-value-coercion . forbidden)))
