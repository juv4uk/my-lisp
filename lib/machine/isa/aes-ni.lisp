; Intel AES New Instructions catalogue. Physical ISA facts only.
; Runtime selection is gated by the CPUID AES feature bit.

(isa-catalogue/1
  (extension AES-NI)
  (source intel-sdm/intel-xed)
  (runtime-gate cpuid-aes)
  (coverage seed)
  (instructions
    (instruction AESENC
      (class cryptographic-round)
      (feature AES-NI)
      (privilege user))
    (instruction AESDEC
      (class cryptographic-round)
      (feature AES-NI)
      (privilege user))
    (instruction AESKEYGENASSIST
      (class cryptographic-key-schedule)
      (feature AES-NI)
      (privilege user))))
