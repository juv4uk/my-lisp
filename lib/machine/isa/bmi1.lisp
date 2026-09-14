; BMI1 instruction-family catalogue. Physical ISA facts only.
; Exact VEX fields stay sourced from XED until the Lisp encoder admits a form.

(isa-catalogue/1
  (extension BMI1)
  (source intel-sdm/intel-xed)
  (runtime-gate cpuid-bmi1)
  (coverage identity-seed)
  (instructions
    (instruction ANDN
      (class bit-manipulation)
      (feature BMI1)
      (privilege user)
      (encoding-authority intel-xed))
    (instruction BEXTR
      (class bit-field-extract)
      (feature BMI1)
      (privilege user)
      (encoding-authority intel-xed))
    (instruction BLSI
      (class bit-isolate)
      (feature BMI1)
      (privilege user)
      (encoding-authority intel-xed))
    (instruction BLSMSK
      (class bit-mask)
      (feature BMI1)
      (privilege user)
      (encoding-authority intel-xed))
    (instruction BLSR
      (class bit-reset)
      (feature BMI1)
      (privilege user)
      (encoding-authority intel-xed))))
