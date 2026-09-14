; Concrete CPU capability profile for the user's Intel Core i5-6400 (Skylake).
; This is hardware capability data, not a semantic registry.
; Static SKU facts come from Intel ARK / Intel SDM.  Features whose actual use
; also depends on process state remain runtime-gated by CPUID/XGETBV/OS support.

(cpu-profile/1
  (cpu intel-core-i5-6400)
  (microarchitecture skylake)
  (isa x86-64)
  (mode 64-bit)

  ; Architectural baseline available in 64-bit user mode.
  (supported-extension X86-BASE)
  (supported-extension X86-64)
  (supported-extension X87)
  (supported-extension MMX)
  (supported-extension SSE)
  (supported-extension SSE2)
  (supported-extension SSE3)
  (supported-extension SSSE3)
  (supported-extension SSE4.1)
  (supported-extension SSE4.2)

  ; Vector/crypto/bit-manipulation families present on Skylake client.
  ; Runtime code must still verify the named CPUID/XGETBV gate before use.
  (gated-extension AES (gate cpuid-aes))
  (gated-extension PCLMULQDQ (gate cpuid-pclmulqdq))
  (gated-extension AVX (gate cpuid-avx+osxsave+xgetbv-xmm-ymm))
  (gated-extension F16C (gate cpuid-f16c+avx-state))
  (gated-extension FMA (gate cpuid-fma+avx-state))
  (gated-extension BMI1 (gate cpuid-bmi1))
  (gated-extension BMI2 (gate cpuid-bmi2))
  (gated-extension AVX2 (gate cpuid-avx2+avx-state))
  (gated-extension RDRAND (gate cpuid-rdrand))
  (gated-extension RDSEED (gate cpuid-rdseed))
  (gated-extension ADX (gate cpuid-adx))
  (gated-extension XSAVE (gate cpuid-xsave))
  (gated-extension CLFLUSHOPT (gate cpuid-clflushopt))

  ; SKU-advertised facilities with additional platform/OS constraints.
  (platform-gated-extension MPX (gate cpuid-mpx+os-support))
  (platform-gated-extension SGX (gate cpuid-sgx+firmware+os-support))

  ; Intel ARK explicitly reports transactional synchronization as unavailable.
  (unavailable-extension TSX)

  (execution-policy
    (ordinary-user-instructions user-mode)
    (privileged-instructions forbidden)
    (runtime-feature-check required)
    (avx-state-check xgetbv-required)))
