; Carry-less multiplication instruction-family catalogue. Physical ISA facts only.

(isa-catalogue/1
  (extension PCLMULQDQ)
  (source intel-sdm/intel-xed)
  (runtime-gate cpuid-pclmulqdq)
  (coverage seed)
  (instructions
    (instruction PCLMULQDQ
      (class carry-less-multiply)
      (feature PCLMULQDQ)
      (privilege user))))
