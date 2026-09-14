; RDRAND instruction-family catalogue. Physical ISA facts only.
; Runtime selection is gated by the CPUID RDRAND feature bit.

(isa-catalogue/1
  (extension RDRAND)
  (source intel-sdm/intel-xed)
  (runtime-gate cpuid-rdrand)
  (coverage seed)
  (instructions
    (instruction RDRAND
      (class hardware-random)
      (feature RDRAND)
      (privilege user)
      (forms
        (form rdrand-r16/r32/r64
          (opcode-map 0F)
          (opcode C7)
          (modrm-reg 6))))))
