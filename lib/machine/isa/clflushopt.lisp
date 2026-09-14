; Optimized cache-line flush instruction-family catalogue. Physical ISA facts only.

(isa-catalogue/1
  (extension CLFLUSHOPT)
  (source intel-sdm/intel-xed)
  (runtime-gate cpuid-clflushopt)
  (coverage seed)
  (instructions
    (instruction CLFLUSHOPT
      (class cache-line-flush)
      (feature CLFLUSHOPT)
      (privilege user))))
