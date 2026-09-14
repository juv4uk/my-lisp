; Intel Core i5-6400 / Skylake execution projection for existing my-lisp identities.
;
; This file is NOT a semantic registry and does not create language meaning.
; `lib/surface/semantic-registry.lisp` remains the only authority for semantic
; identities and human surfaces.  Each row here is keyed by an existing
; semantic ID and records a plausible physical realization on this processor.
;
; The projection deliberately distinguishes a one/few-instruction fast path
; from a wider Lisp/runtime implementation.  For example `+` keeps arbitrary-
; precision exact semantics even though small exact integers can use ADD.
;
; Row schema:
;   (semantic-id class "machine path")
;
; class is descriptive only:
;   direct    — one/few ordinary ISA operations are the natural realization
;   fast-path — ISA operation covers a bounded representation; Lisp fallback remains
;   sequence  — several ordinary ISA operations implement the semantic action
;   runtime   — allocation/iteration/runtime state is required
;   control   — maps naturally to CPU control flow

(machine-profile/1
  (cpu intel-core-i5-6400)
  (microarchitecture skylake)
  (isa x86-64)
  (rows
    (0002 sequence "tag-test: TEST/AND/CMP")
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
    (1140 control "short-circuit TEST/Jcc")
  ))
