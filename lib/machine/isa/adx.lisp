; Multi-precision add-carry instruction-family catalogue. Physical ISA facts only.

(isa-catalogue/1
  (extension ADX)
  (source intel-sdm/intel-xed)
  (runtime-gate cpuid-adx)
  (coverage seed)
  (instructions
    (instruction ADCX
      (class add-carry)
      (feature ADX)
      (privilege user))
    (instruction ADOX
      (class add-overflow)
      (feature ADX)
      (privilege user))))
