; mathematical-result-taxonomy.lisp — Lisp-owned mathematical result authority.
;
; #225 does not invent a replacement mathematics. It preserves ordinary
; mathematical classification and keeps that classification separate from the
; role/representation of a result.
;
; Two independent questions must not be collapsed:
;
;   1. What mathematical object is this?
;      Example: pi is real, irrational, transcendental.
;
;   2. How is this result representing or relating to that object?
;      Example: 314159/100000 is itself an exact rational number, and it may
;      additionally play the role of a rational approximation to pi.
;
; Core law:
;   mathematical object != its approximation
;
; Non-binary mathematics remains mathematics. Nothing in this document is a
; many-valued truth state, and no entry authorizes generic Value -> bool.

(mathematical-result-taxonomy/1
  ((identity . laws)
   (domain-owner . mathematical-result)
   (classification-source . standard-mathematics)
   (object-approximation-identity . distinct)
   (generic-truth-coercion . forbidden)
   (many-valued-collapse . forbidden)
   (lower-logic-redefinition . forbidden))

  ; A rational result is already an exact mathematical object. The fact that
  ; the same rational may be used as an approximation of another object is a
  ; separate result-role relation, not a change of the rational's identity.
  ((identity . rational-example)
   (value . 1/2)
   (number-system . rational)
   (subset-of . real)
   (result-role . exact-object))

  ; pi is not an approximation. It is the exact mathematical constant; a
  ; runtime may later choose symbolic, exact-real, or other representations.
  ((identity . pi-example)
   (object . pi)
   (number-system . real)
   (rationality . irrational)
   (algebraic-status . transcendental)
   (result-role . exact-object)
   (representation-policy . representation-may-vary-without-changing-object))

  ; sqrt(2) is an exact algebraic irrational mathematical object even when a
  ; concrete runtime represents it symbolically rather than as a finite scalar.
  ((identity . sqrt2-example)
   (object . (sqrt 2))
   (number-system . real)
   (rationality . irrational)
   (algebraic-status . algebraic)
   (result-role . exact-object)
   (representation-policy . representation-may-vary-without-changing-object))

  ; An approximation relates two mathematical objects: the exact approximant
  ; and its target. It must state how strongly the approximation is justified.
  ((identity . approximation-role)
   (result-role . approximation)
   (required-metadata . (target approximant precision-or-error-guarantee))
   (approximant-remains-own-mathematical-object . t)
   (truth-status . not-applicable))

  ; An interval/enclosure is mathematical data. Its guarantee states that the
  ; target lies within the represented bounds; it is not a fuzzy truth degree.
  ((identity . interval-enclosure-role)
   (result-role . interval-enclosure)
   (required-metadata . (target lower-bound upper-bound enclosure-guarantee))
   (truth-status . not-applicable))

  ; Symbolic form is a representation role. A symbolic expression may denote
  ; an exact mathematical object (pi, sqrt(2), an unevaluated algebraic form)
  ; without becoming an epistemic or many-valued status.
  ((identity . symbolic-expression-role)
   (result-role . symbolic-expression)
   (may-denote-exact-mathematical-object . t)
   (many-valued-status . forbidden)))
