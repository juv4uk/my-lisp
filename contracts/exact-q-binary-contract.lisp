; exact-q-binary-contract.lisp — Lisp-owned mathematical binary domain for #216.
;
; This is not universal truth. It is the answer algebra of one narrow domain:
; comparisons whose required operands are all established exactly in Q.
;
; Mathematical outcomes are the exact rationals 0/1 (NO) and 1/1 (YES).
; The ordinary printer may canonically render denominator-1 rationals as 0/1
; -> "0" and 1/1 -> "1"; spelling is not the semantic value.
;
; Outside this domain the layer is silent: (). That means no answer from this
; layer, never mathematical NO. Control over such results belongs to #217.

(exact-q-binary-contract/1
  ((domain-owner . exact-q-decision)
   (admissibility . all-required-values-exact-rational)
   (no-answer . ())
   (outside-domain . no-answer)
   (approximation-policy . forbidden)
   (generic-truth-coercion . forbidden)
   (control-dispatch . delegated-to-217)
   (outcomes .
     (((meaning . no)
       (numerator . 0)
       (denominator . 1)
       (canonical-write . "0"))
      ((meaning . yes)
       (numerator . 1)
       (denominator . 1)
       (canonical-write . "1")))))

  ((identity . "1014")
   (surface . <)
   (relation . strictly-increasing)
   (operand-domain . exact-rational-sequence))

  ((identity . "1015")
   (surface . >)
   (relation . strictly-decreasing)
   (operand-domain . exact-rational-sequence))

  ((identity . "1016")
   (surface . =)
   (relation . numeric-equality)
   (operand-domain . exact-rational-sequence))

  ((identity . "1017")
   (surface . <=)
   (relation . nondecreasing)
   (operand-domain . exact-rational-sequence))

  ((identity . "1018")
   (surface . >=)
   (relation . nonincreasing)
   (operand-domain . exact-rational-sequence)))
