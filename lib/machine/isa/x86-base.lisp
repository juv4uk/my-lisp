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
          (modrm required))
        (form test-r64-imm32
          (operands r/m64 imm32)
          (rex W)
          (opcode F7)
          (reg-opcode 0)
          (immediate-width 32)
          (modrm required))))

    (instruction CALL
      (class control-transfer)
      (privilege user)
      (mode64 valid)
      (forms
        (form call-rel32
          (operands rel32)
          (opcode E8)
          (immediate-width 32))
        (form call-r64
          (operands r64)
          (opcode FF)
          (reg-opcode 2)
          (modrm required))))

    (instruction JMP
      (class control-transfer)
      (privilege user)
      (mode64 valid)
      (forms
        (form jmp-rel32
          (operands rel32)
          (opcode E9)
          (immediate-width 32))
        (form jmp-r64
          (operands r64)
          (opcode FF)
          (reg-opcode 4)
          (modrm required))))

    (instruction OR
      (class logical-operation)
      (privilege user)
      (mode64 valid)
      (flags OF SF ZF PF CF)
      (forms
        (form or-r64-r64
          (operands r/m64 r64)
          (rex W)
          (opcode 09)
          (modrm required))
        (form or-r64-imm32
          (operands r/m64 imm32)
          (rex W)
          (opcode 81)
          (reg-opcode 1)
          (immediate-width 32)
          (modrm required))))

    (instruction AND
      (class logical-operation)
      (privilege user)
      (mode64 valid)
      (flags OF SF ZF PF CF)
      (forms
        (form and-r64-r64
          (operands r/m64 r64)
          (rex W)
          (opcode 21)
          (modrm required))
        (form and-r64-imm32
          (operands r/m64 imm32)
          (rex W)
          (opcode 81)
          (reg-opcode 4)
          (immediate-width 32)
          (modrm required))))

    (instruction XOR
      (class logical-operation)
      (privilege user)
      (mode64 valid)
      (flags OF SF ZF PF CF)
      (forms
        (form xor-r64-r64
          (operands r/m64 r64)
          (rex W)
          (opcode 31)
          (modrm required))
        (form xor-r64-imm32
          (operands r/m64 imm32)
          (rex W)
          (opcode 81)
          (reg-opcode 6)
          (immediate-width 32)
          (modrm required))))

    (instruction PUSH
      (class stack-operation)
      (privilege user)
      (mode64 valid)
      (forms
        (form push-r64
          (operands r64)
          (opcode-plus-register 50)
          (modrm none))))

    (instruction POP
      (class stack-operation)
      (privilege user)
      (mode64 valid)
      (forms
        (form pop-r64
          (operands r64)
          (opcode-plus-register 58)
          (modrm none))))

    (instruction INC
      (class integer-arithmetic)
      (privilege user)
      (mode64 valid)
      (flags OF SF ZF AF PF)
      (forms
        (form inc-r64
          (operands r/m64)
          (rex W)
          (opcode FF)
          (reg-opcode 0)
          (modrm required))))

    (instruction DEC
      (class integer-arithmetic)
      (privilege user)
      (mode64 valid)
      (flags OF SF ZF AF PF)
      (forms
        (form dec-r64
          (operands r/m64)
          (rex W)
          (opcode FF)
          (reg-opcode 1)
          (modrm required))))

    (instruction NOT
      (class logical-operation)
      (privilege user)
      (mode64 valid)
      (forms
        (form not-r64
          (operands r/m64)
          (rex W)
          (opcode F7)
          (reg-opcode 2)
          (modrm required))))

    (instruction NEG
      (class integer-arithmetic)
      (privilege user)
      (mode64 valid)
      (flags CF OF SF ZF AF PF)
      (forms
        (form neg-r64
          (operands r/m64)
          (rex W)
          (opcode F7)
          (reg-opcode 3)
          (modrm required))))

    (instruction SHL
      (class shift-operation)
      (privilege user)
      (mode64 valid)
      (flags CF OF SF ZF AF PF)
      (forms
        (form shl-r64-imm8
          (operands r/m64 imm8)
          (rex W)
          (opcode C1)
          (reg-opcode 4)
          (immediate-width 8)
          (modrm required))))

    (instruction SHR
      (class shift-operation)
      (privilege user)
      (mode64 valid)
      (flags CF OF SF ZF AF PF)
      (forms
        (form shr-r64-imm8
          (operands r/m64 imm8)
          (rex W)
          (opcode C1)
          (reg-opcode 5)
          (immediate-width 8)
          (modrm required))))

    (instruction SAR
      (class shift-operation)
      (privilege user)
      (mode64 valid)
      (flags CF OF SF ZF AF PF)
      (forms
        (form sar-r64-imm8
          (operands r/m64 imm8)
          (rex W)
          (opcode C1)
          (reg-opcode 7)
          (immediate-width 8)
          (modrm required))))

    (instruction LEA
      (class address-generation)
      (privilege user)
      (mode64 valid)
      (forms
        (form lea-r64-mem-disp8
          (operands r64 m)
          (rex W)
          (opcode 8D)
          (modrm required))))

    (instruction JCC
      (class conditional-branch)
      (privilege user)
      (mode64 valid)
      (forms
        (form jcc-rel8
          (operands rel8)
          (opcode-range 70 7F)
          (immediate-width 8))
        (form jcc-rel32
          (operands rel32)
          (opcode-map 0F)
          (opcode-range 80 8F)
          (immediate-width 32))))

    (instruction SETCC
      (class conditional-byte-set)
      (privilege user)
      (mode64 valid)
      (forms
        (form setcc-r8
          (operands r/m8)
          (opcode-map 0F)
          (opcode-range 90 9F)
          (reg-opcode 0)
          (modrm required))))

    (instruction MOVZX
      (class data-movement-zero-extend)
      (privilege user)
      (mode64 valid)
      (forms
        (form movzx-r64-r8
          (operands r64 r/m8)
          (rex W)
          (opcode-map 0F)
          (opcode B6)
          (modrm required))))

    (instruction IMUL
      (class integer-multiply)
      (privilege user)
      (mode64 valid)
      (flags CF OF)
      (forms
        (form imul-r64-r64
          (operands r64 r/m64)
          (rex W)
          (opcode-map 0F)
          (opcode AF)
          (modrm required))
        (form imul-r64-r64-imm32
          (operands r64 r/m64 imm32)
          (rex W)
          (opcode 69)
          (immediate-width 32)
          (modrm required))))

    (instruction CQO
      (class integer-sign-extend)
      (privilege user)
      (mode64 valid)
      (forms
        (form cqo
          (operands)
          (rex W)
          (opcode 99)
          (modrm none))))

    (instruction IDIV
      (class integer-divide)
      (privilege user)
      (mode64 valid)
      (flags CF OF SF ZF AF PF)
      (forms
        (form idiv-r64
          (operands r/m64)
          (rex W)
          (opcode F7)
          (reg-opcode 7)
          (modrm required))))

    (instruction CMOVCC
      (class conditional-data-movement)
      (privilege user)
      (mode64 valid)
      (forms
        (form cmovcc-r64-r64
          (operands r64 r/m64)
          (rex W)
          (opcode-map 0F)
          (opcode-range 40 4F)
          (modrm required))))

    (instruction NOP
      (class no-operation)
      (privilege user)
      (mode64 valid)
      (forms
        (form nop
          (operands)
          (opcode 90)
          (modrm none))))

    (instruction BT
      (class bit-test)
      (privilege user)
      (mode64 valid)
      (flags CF)
      (forms
        (form bt-r64-r64
          (operands r/m64 r64)
          (rex W)
          (opcode-map 0F)
          (opcode A3)
          (modrm required))
        (form bt-r64-imm8
          (operands r/m64 imm8)
          (rex W)
          (opcode-map 0F)
          (opcode BA)
          (reg-opcode 4)
          (immediate-width 8)
          (modrm required))))

    (instruction BTS
      (class bit-test-and-set)
      (privilege user)
      (mode64 valid)
      (flags CF)
      (forms
        (form bts-r64-imm8
          (operands r/m64 imm8)
          (rex W)
          (opcode-map 0F)
          (opcode BA)
          (reg-opcode 5)
          (immediate-width 8)
          (modrm required))))

    (instruction BTR
      (class bit-test-and-reset)
      (privilege user)
      (mode64 valid)
      (flags CF)
      (forms
        (form btr-r64-imm8
          (operands r/m64 imm8)
          (rex W)
          (opcode-map 0F)
          (opcode BA)
          (reg-opcode 6)
          (immediate-width 8)
          (modrm required))))

    (instruction BTC
      (class bit-test-and-complement)
      (privilege user)
      (mode64 valid)
      (flags CF)
      (forms
        (form btc-r64-imm8
          (operands r/m64 imm8)
          (rex W)
          (opcode-map 0F)
          (opcode BA)
          (reg-opcode 7)
          (immediate-width 8)
          (modrm required))))

    (instruction MOVSX
      (class data-movement-sign-extend)
      (privilege user)
      (mode64 valid)
      (forms
        (form movsx-r64-r8
          (operands r64 r/m8)
          (rex W)
          (opcode-map 0F)
          (opcode BE)
          (modrm required))))

    (instruction MOVSXD
      (class data-movement-sign-extend)
      (privilege user)
      (mode64 valid)
      (forms
        (form movsxd-r64-r32
          (operands r64 r/m32)
          (rex W)
          (opcode 63)
          (modrm required))))))
