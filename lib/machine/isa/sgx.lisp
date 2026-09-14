; Intel SGX instruction-family catalogue. Physical ISA facts only.
; Availability on this SKU is platform-gated by CPUID, firmware/ME, and OS support.

(isa-catalogue/1
  (extension SGX)
  (source intel-sdm/intel-xed)
  (runtime-gate cpuid-sgx+firmware+os-support)
  (coverage seed)
  (instructions
    (instruction ENCLU
      (class enclave-user-leaf)
      (feature SGX)
      (privilege user))
    (instruction ENCLS
      (class enclave-supervisor-leaf)
      (feature SGX)
      (privilege supervisor))))
