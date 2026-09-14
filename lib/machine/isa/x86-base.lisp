; x86 base ISA catalogue — physical machine facts, not my-lisp semantics.
; Authority: Intel SDM / Intel XED.  No row in this file may allocate a
; my-lisp semantic identity.

(isa-catalogue/1
  (extension X86-BASE)
  (architecture x86)
  (source intel-sdm/intel-xed)
  (instructions
    (instruction MOV
      (class data-movement)
      (privilege user)
      (mode64 valid)
      (forms
        (form mov-r32-imm32
          (operands r32 imm32)
          (opcode-plus-register B8)
          (immediate-width 32)
          (modrm none))
        (form mov-r64-r64
          (operands r64 r64)
          (rex W)
          (opcode 89)
          (modrm required))))

    (instruction ADD
      (class integer-arithmetic)
      (privilege user)
      (mode64 valid)
      (flags OF SF ZF AF CF PF)
      (forms
        (form add-r64-r64
          (operands r/m64 r64)
          (rex W)
          (opcode 01)
          (modrm required)
          (proof-example
            (operands RAX RBX)
            (bytes 48 01 D8)))))

    (instruction RET
      (class control-transfer)
      (privilege user)
      (mode64 valid)
      (forms
        (form ret-near
          (operands)
          (opcode C3)
          (modrm none)
          (bytes C3))))

    (instruction SUB
      (class integer-arithmetic)
      (privilege user)
      (mode64 valid)
      (forms
        (form sub-r64-r64
          (operands r/m64 r64)
          (rex W)
          (opcode 29)
          (modrm required))))

    (instruction CMP
      (class integer-compare)
      (privilege user)
      (mode64 valid)
      (forms
        (form cmp-r64-r64
          (operands r/m64 r64)
          (rex W)
          (opcode 39)
          (modrm required))))

    (instruction TEST
      (class bit-test)
      (privilege user)
      (mode64 valid)
      (forms
        (form test-r64-r64
          (operands r/m64 r64)
          (rex W)
          (opcode 85)
          (modrm required))))

    (instruction CALL
      (class control-transfer)
      (privilege user)
      (mode64 valid)
      (forms
        (form call-rel32
          (operands rel32)
          (opcode E8)
          (immediate-width 32))))

    (instruction JMP
      (class control-transfer)
      (privilege user)
      (mode64 valid)
      (forms
        (form jmp-rel32
          (operands rel32)
          (opcode E9)
          (immediate-width 32))))))
