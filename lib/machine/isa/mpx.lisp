; Intel Memory Protection Extensions catalogue. Physical ISA facts only.
; Availability is platform/OS gated even when the processor advertises MPX.

(isa-catalogue/1
  (extension MPX)
  (source intel-sdm/intel-xed)
  (runtime-gate cpuid-mpx+os-support)
  (coverage seed)
  (instructions
    (instruction BNDCL
      (class bounds-check-lower)
      (feature MPX)
      (privilege user))
    (instruction BNDCU
      (class bounds-check-upper)
      (feature MPX)
      (privilege user))
    (instruction BNDMK
      (class bounds-create)
      (feature MPX)
      (privilege user))
    (instruction BNDMOV
      (class bounds-move)
      (feature MPX)
      (privilege user))))
