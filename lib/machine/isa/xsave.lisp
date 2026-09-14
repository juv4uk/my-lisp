; Extended processor-state save/restore instruction-family catalogue.
; Physical ISA facts only; usable state components are runtime/OS gated.

(isa-catalogue/1
  (extension XSAVE)
  (source intel-sdm/intel-xed)
  (runtime-gate cpuid-xsave)
  (coverage seed)
  (instructions
    (instruction XSAVE
      (class processor-state-save)
      (feature XSAVE)
      (privilege user))
    (instruction XRSTOR
      (class processor-state-restore)
      (feature XSAVE)
      (privilege user))))
