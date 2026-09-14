; MMX instruction-family catalogue. Physical ISA facts only.
; MMX is retained on the i5-6400 for architectural compatibility.

(isa-catalogue/1
  (extension MMX)
  (source intel-sdm/intel-xed)
  (coverage seed)
  (instructions
    (instruction EMMS
      (class mmx-state)
      (feature MMX)
      (privilege user)
      (forms
        (form emms
          (operands)
          (opcode-map 0F)
          (opcode 77))))
    (instruction MOVQ
      (class packed-move)
      (feature MMX)
      (privilege user))))
