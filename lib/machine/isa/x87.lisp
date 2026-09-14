; x87 floating-point instruction-family catalogue. Physical ISA facts only.

(isa-catalogue/1
  (extension X87)
  (source intel-sdm/intel-xed)
  (coverage seed)
  (instructions
    (instruction FLD
      (class floating-load)
      (feature X87)
      (privilege user))
    (instruction FSTP
      (class floating-store)
      (feature X87)
      (privilege user))
    (instruction FADD
      (class floating-arithmetic)
      (feature X87)
      (privilege user))))
