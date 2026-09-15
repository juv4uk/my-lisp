; Semantic -> x86-64 lowering projection.
; Language meaning remains owned by lib/surface/semantic-registry.lisp.
; ISA identity/encoding remains owned by lib/machine/isa + lib/machine/encoding.
; Rows here only say how an already-existing semantic identity may be realized.
;
; Executable lowerings produce structured machine forms. Byte materialization
; belongs to the closed Lisp-owned admission layer and is deliberately not
; re-exported from semantic lowering as a compatibility convenience.

(def x86-semantic-lowering-profile
  (quote
    ((0002 sequence "tag-test: TEST/AND/CMP")
     (0003 direct "CMP/SETE")
     (0004 runtime "allocate+STORE-pair")
     (0005 direct "LOAD-pair-head")
     (0006 direct "LOAD-pair-tail")
     (0007 control "TEST/CMP+Jcc")

     (0104 fast-path "ADD / ADDSD")
     (1001 fast-path "SUB/NEG / SUBSD")
     (1002 fast-path "IMUL / MULSD")
     (1003 fast-path "DIVSD; exact-rational routine")
     (1004 sequence "TEST/NEG/CMOV")
     (1005 sequence "CMP/CMOV-min")
     (1006 sequence "CMP/CMOV-max")
     (1007 fast-path "IDIV-remainder")
     (1008 fast-path "IDIV-quotient")
     (1009 fast-path "SQRTSD")
     (1010 runtime "integer-sqrt routine")
     (1011 runtime "loop+CMP/CMOV-min")
     (1012 runtime "loop+CMP/CMOV-max")
     (1014 direct "CMP/SETL")
     (1015 direct "CMP/SETG")
     (1016 direct "CMP/SETE")
     (1017 direct "CMP/SETLE")
     (1018 direct "CMP/SETGE")
     (1021 direct "TEST/SETE")

     (1023 sequence "tag-test: symbol")
     (1024 sequence "tag-test: string")
     (1026 sequence "tag-test: numeric-buffer")
     (1028 runtime "list-walk+LOAD-pair-tail")
     (1031 runtime "indexed-list-walk+LOAD")
     (1035 sequence "LOAD-tail+LOAD-head")
     (1036 sequence "2xLOAD-tail+LOAD-head")
     (1037 sequence "3xLOAD-tail+LOAD-head")
     (1038 sequence "4xLOAD-tail+LOAD-head")
     (1039 sequence "LOAD-head+LOAD-head")
     (1040 sequence "LOAD-tail+LOAD-head")
     (1041 sequence "LOAD-tail+LOAD-tail")
     (1042 sequence "3xLOAD-tail+LOAD-head")

     (1064 runtime "allocate+STORE-vector")
     (1065 runtime "allocate+fill-vector")
     (1066 direct "LOAD-vector-length")
     (1067 direct "LOAD-vector-element")
     (1068 direct "STORE-vector-element")
     (1069 runtime "allocate+STORE-i32-buffer")
     (1070 runtime "allocate+STORE-f32-buffer")
     (1071 direct "LOAD-buffer-tag")
     (1072 direct "LOAD-buffer-length")
     (1073 direct "LOAD-buffer-element")
     (1074 runtime "loop; AVX2 specialization possible")

     (1082 fast-path "IDIV/IMUL-reciprocal")
     (1087 direct "CMP/SETGE")
     (1088 direct "CMP/SETGE")
     (1089 direct "SUB")
     (1090 direct "ADD")
     (1091 direct "ADD")

     (1098 runtime "empty-vector object")
     (1099 runtime "allocate+copy+STORE-vector")
     (1100 direct "LOAD-vector-length")
     (1101 direct "LOAD-vector-element")

     (1137 direct "MOV/pass-through")
     (1139 control "short-circuit TEST/Jcc")
     (1140 control "short-circuit TEST/Jcc"))))

; First executable semantic-lowering witness.
; Semantic identity 0104 already exists before this file is loaded. This
; routine does not define addition; it chooses one bounded u64 realization.
; It returns structured machine forms only. Admission owns the path from those
; forms to executable bytes.
;
; RAX carries the result per SysV x86-64. RCX is caller-saved, so the proof
; routine does not violate the host ABI by clobbering a callee-saved register.
(def x86-lower-add-u64-forms
  (lambda (left right)
    (list
      (list (quote mov-r64-imm64) (quote rax) left)
      (list (quote mov-r64-imm64) (quote rcx) right)
      (list (quote add-r64-r64) (quote rax) (quote rcx))
      (list (quote ret)))))

; #196 bounded conditional-growth witness for existing EQ + COND semantics.
; This routine does not define equality or conditional evaluation. It chooses
; one fixed-width u64 realization whose only purpose is to establish the
; machine-effect lower bound demanded by a two-arm runtime decision.
;
; JNZ +11 skips exactly one MOV r64,imm64 (10 bytes) plus one RET (1 byte),
; landing at the ELSE arm. This is deliberately not a general label resolver,
; branch assembler, or compiler policy. It returns structured forms only;
; closed admission remains the sole path to bytes.
(def x86-lower-eq-cond-u64-forms
  (lambda (left right then-value else-value)
    (list
      (list (quote mov-r64-imm64) (quote rax) left)
      (list (quote mov-r64-imm64) (quote rcx) right)
      (list (quote cmp-r64-r64) (quote rax) (quote rcx))
      (list (quote jnz-rel8) 11)
      (list (quote mov-r64-imm64) (quote rax) then-value)
      (list (quote ret))
      (list (quote mov-r64-imm64) (quote rax) else-value)
      (list (quote ret)))))

; Bounded structural witness for semantic identities 0004/0005/0006.
; The host contributes only a raw writable arena pointer in RDI. Lisp owns
; the fact that one admitted pair cell has head at x86-pair-car-offset and
; tail at x86-pair-cdr-offset. These routines produce structured STORE/LOAD
; forms; admission and the encoder produce physical bytes only afterwards.
;
; This is deliberately not a claim that arbitrary first-class pair values may
; already escape native code: pair-x86-64.lisp fixes lifetime=native-call and
; escape=forbidden for this proof slice.
(def x86-lower-bounded-pair-store-u64-forms
  (lambda (left right)
    (list
      (list (quote mov-r64-imm64) (quote rax) left)
      (list
        (quote mov-mem-disp8-r64)
        (quote rdi)
        x86-pair-car-offset
        (quote rax))
      (list (quote mov-r64-imm64) (quote rax) right)
      (list
        (quote mov-mem-disp8-r64)
        (quote rdi)
        x86-pair-cdr-offset
        (quote rax)))))

; Historical internal name retained only as a form-level alias so existing
; Lisp callers do not regain a byte-level bypass.
(def x86-lower-bounded-pair-store-u64-instructions
  x86-lower-bounded-pair-store-u64-forms)

(def x86-lower-cons-car-u64-forms
  (lambda (left right)
    (append
      (x86-lower-bounded-pair-store-u64-forms left right)
      (list
        (list
          (quote mov-r64-mem-disp8)
          (quote rax)
          (quote rdi)
          x86-pair-car-offset)
        (list (quote ret))))))

(def x86-lower-cons-cdr-u64-forms
  (lambda (left right)
    (append
      (x86-lower-bounded-pair-store-u64-forms left right)
      (list
        (list
          (quote mov-r64-mem-disp8)
          (quote rax)
          (quote rdi)
          x86-pair-cdr-offset)
        (list (quote ret))))))

; Bounded semantic entry for the Vertical Day CAR witness.
; Canonical CAR/CDR own pair validity and therefore fail with the language's
; existing Type outcome before any machine request exists. Only after that
; language-owned gate succeeds do the extracted u64 fields become structured
; machine forms, pass closed admission, and enter the semantics-blind host.
; No pair predicate or tag rule is duplicated in this machine layer.
(def x86-call-semantic-car-u64
  (lambda (pair-value)
    (x86-call-admitted-u64
      (x86-lower-cons-car-u64-forms
        (car pair-value)
        (cdr pair-value))
      x86-pair-cell-bytes)))
