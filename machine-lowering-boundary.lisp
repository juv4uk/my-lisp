; Machine-readable authority boundary between language meaning and physical target facts.
; This file deliberately contains no target mnemonic, register, opcode, or ISA name.
; my-lisp owns language meaning and semantic IDs. Hardware specifications own ISA
; facts. my-lisp may represent and encode those facts. CML remains an optimizer,
; while the host owns only raw executable-memory / invocation mechanisms.

(machine-lowering-boundary
  (schema machine-lowering-boundary/2)
  (semantic-authority my-lisp)
  (isa-authority hardware-specification)
  (isa-source intel-xed/intel-sdm)
  (isa-representation my-lisp)
  (instruction-encoding my-lisp)
  (optimization-authority cml)
  (portable-monotonic-observation 1075)
  (semantic-id-allocation explicit-language-contract-only)
  (semantic-id-from-isa forbidden)
  (lowering-direction semantic-to-machine)
  (reverse-authority forbidden)
  (raw-execution-mechanism host)
  (machine-text projection-only)
  (machine-bytes lisp-owned-target-product)
  ; 1153 briefly reached main from a compiler-originated experiment. It is burned:
  ; never assign it to another language meaning, while keeping it out of the active
  ; semantic registry because no language contract admitted that operation.
  (retired-semantic-id 1153))
