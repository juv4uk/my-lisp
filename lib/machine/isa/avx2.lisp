; AVX2 instruction-family catalogue. Physical ISA facts only.
; Actual execution remains gated by CPUID + OS-enabled YMM state.

(isa-catalogue/1
  (extension AVX2)
  (requires AVX)
  (source intel-sdm/intel-xed)
  (runtime-gate cpuid-avx2+avx-state)
  (coverage seed)
  (instructions
    (instruction VPADDD
      (class packed-integer-arithmetic)
      (feature AVX2)
      (privilege user)
      (forms
        (form vpaddd-ymm-ymm-ymmm256
          (operands ymm ymm ymm/m256)
          (encoding VEX)
          (mandatory-prefix 66)
          (opcode-map 0F)
          (opcode FE)
          (modrm required))))
    (instruction VPMULLD
      (class packed-integer-arithmetic)
      (feature AVX2)
      (privilege user)
      (forms
        (form vpmulld-ymm-ymm-ymmm256
          (operands ymm ymm ymm/m256)
          (encoding VEX)
          (mandatory-prefix 66)
          (opcode-map 0F38)
          (opcode 40)
          (modrm required))))))
