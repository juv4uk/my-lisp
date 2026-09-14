; Lisp-owned x86 assembler surface.
; The language owns instruction constructors and assembly orchestration.
; Physical x86 encoding will be delegated to a mature assembler tool; no
; opcode/REX/ModR/M table belongs here or in the Rust host.

(def x86-ret
  (lambda ()
    "ret\n"))

; First TDD slice: establish the public assembler operation as an ordinary
; Lisp closure. The next contract will replace this identity body with the
; external assembler pipeline while keeping the same Lisp-owned boundary.
(def x86-assemble
  (lambda (program)
    program))
