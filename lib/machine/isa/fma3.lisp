; FMA3 instruction-family catalogue. Physical ISA facts only.
; Execution requires AVX register state plus the CPUID FMA feature bit.

(isa-catalogue/1
  (extension FMA3)
  (requires AVX)
  (source intel-sdm/intel-xed)
  (runtime-gate cpuid-fma+avx-state)
  (coverage seed)
  (instructions
    (instruction VFMADD132PS
      (class fused-floating-arithmetic)
      (feature FMA3)
      (privilege user))
    (instruction VFMADD213PS
      (class fused-floating-arithmetic)
      (feature FMA3)
      (privilege user))
    (instruction VFMADD231PS
      (class fused-floating-arithmetic)
      (feature FMA3)
      (privilege user))))
