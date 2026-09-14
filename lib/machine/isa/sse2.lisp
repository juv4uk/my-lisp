; SSE2 instruction-family catalogue. Physical ISA facts only.

(isa-catalogue/1
  (extension SSE2)
  (requires SSE)
  (source intel-sdm/intel-xed)
  (coverage seed)
  (instructions
    (instruction ADDSD
      (class scalar-floating-point)
      (feature SSE2)
      (privilege user)
      (forms
        (form addsd-xmm-xmmm64
          (operands xmm xmm/m64)
          (legacy-prefix F2)
          (opcode-map 0F)
          (opcode 58)
          (modrm required))))
    (instruction MOVSD
      (class scalar-floating-point-data-movement)
      (feature SSE2)
      (privilege user)
      (forms
        (form movsd-xmm-xmmm64
          (operands xmm xmm/m64)
          (legacy-prefix F2)
          (opcode-map 0F)
          (opcode 10)
          (modrm required))))
    (instruction PADDD
      (class packed-integer-arithmetic)
      (feature SSE2)
      (privilege user)
      (forms
        (form paddd-xmm-xmmm128
          (operands xmm xmm/m128)
          (legacy-prefix 66)
          (opcode-map 0F)
          (opcode FE)
          (modrm required))))))
