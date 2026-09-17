(def x86-reg-code
  (lambda (register)
    (cond
      ((eq register (quote rax)) 0)
      ((eq register (quote rcx)) 1)
      ((eq register (quote rdx)) 2)
      ((eq register (quote rbx)) 3)
      ((eq register (quote rsp)) 4)
      ((eq register (quote rbp)) 5)
      ((eq register (quote rsi)) 6)
      ((eq register (quote rdi)) 7)
      ((eq register (quote r8)) 8)
      ((eq register (quote r9)) 9)
      ((eq register (quote r10)) 10)
      ((eq register (quote r11)) 11)
      ((eq register (quote r12)) 12)
      ((eq register (quote r13)) 13)
      ((eq register (quote r14)) 14)
      ((eq register (quote r15)) 15)
      (t (quote ())))))

(def x86-low3
  (lambda (code)
    (mod code 8)))

(def x86-high1
  (lambda (code)
    (quotient code 8)))

(def x86-encode-rex
  (lambda (w r x b)
    (+ 64 (+ (* w 8) (+ (* r 4) (+ (* x 2) b))))))

(def x86-encode-modrm
  (lambda (mode reg rm)
    (+ (* mode 64) (+ (* reg 8) rm))))

(def x86-encode-sib
  (lambda (scale index base)
    (+ (* scale 64) (+ (* index 8) base))))

(def x86-u32-bytes
  (lambda (value)
    (list
      (mod value 256)
      (mod (quotient value 256) 256)
      (mod (quotient value 65536) 256)
      (mod (quotient value 16777216) 256))))

(def x86-u64-bytes
  (lambda (value)
    (list
      (mod value 256)
      (mod (quotient value 256) 256)
      (mod (quotient value 65536) 256)
      (mod (quotient value 16777216) 256)
      (mod (quotient value 4294967296) 256)
      (mod (quotient value 1099511627776) 256)
      (mod (quotient value 281474976710656) 256)
      (mod (quotient value 72057594037927936) 256))))

(def x86-encode-ret
  (lambda ()
    (list 195)))

(def x86-encode-mov-eax-imm32
  (lambda (immediate)
    (cons 184 (x86-u32-bytes immediate))))

(def x86-encode-mov-r64-imm64
  (lambda (register immediate)
    (let ((code (x86-reg-code register)))
      (cons
        (x86-encode-rex 1 0 0 (x86-high1 code))
        (cons
          (+ 184 (x86-low3 code))
          (x86-u64-bytes immediate))))))

; Two's-complement byte for a disp8 value already known to be in [-128,127].
; `mod` in this Lisp does not wrap negative operands (`(mod -1 256)` is -1,
; not 255), so a plain `(mod displacement 256)` silently produced an
; out-of-range byte for any negative displacement -- caught fail-closed at
; the host boundary (`native-call-u64-raw` rejects non-0..255 bytes), but it
; meant negative disp8 could never actually be encoded despite the encoder
; otherwise already supporting arbitrary GPR bases/destinations, SIB for
; rsp/r12, REX.B for an extended base (r8-r15), and REX.R for an extended
; load-destination/store-source register. Adding 256 before reducing mod
; 256 is exact for the whole disp8 domain (verified by round-trip below,
; independently cross-checked against objdump across the full 16x16 base x
; data-register matrix).
(def x86-disp8-byte
  (lambda (displacement)
    (mod (+ displacement 256) 256)))

; MOV r64, [base + disp8], opcode 8B /r.
; ModR/M mode 01 always carries one displacement byte. RSP/R12 bases use the
; required no-index SIB byte instead of silently emitting an invalid address.
(def x86-encode-mov-r64-mem-disp8
  (lambda (destination base displacement)
    (let ((dst (x86-reg-code destination)))
      (let ((base-code (x86-reg-code base)))
        (let ((rex (x86-encode-rex 1 (x86-high1 dst) 0 (x86-high1 base-code))))
          (let ((modrm (x86-encode-modrm 1 (x86-low3 dst) (x86-low3 base-code))))
            (cond
              ((eq (x86-low3 base-code) 4)
                (list
                  rex
                  139
                  modrm
                  (x86-encode-sib 0 4 4)
                  (x86-disp8-byte displacement)))
              (t
                (list rex 139 modrm (x86-disp8-byte displacement))))))))))

; MOV [base + disp8], r64, opcode 89 /r.
(def x86-encode-mov-mem-disp8-r64
  (lambda (base displacement source)
    (let ((base-code (x86-reg-code base)))
      (let ((src (x86-reg-code source)))
        (let ((rex (x86-encode-rex 1 (x86-high1 src) 0 (x86-high1 base-code))))
          (let ((modrm (x86-encode-modrm 1 (x86-low3 src) (x86-low3 base-code))))
            (cond
              ((eq (x86-low3 base-code) 4)
                (list
                  rex
                  137
                  modrm
                  (x86-encode-sib 0 4 4)
                  (x86-disp8-byte displacement)))
              (t
                (list rex 137 modrm (x86-disp8-byte displacement))))))))))

; LEA r64, [base + disp8], opcode 0x8D /r -- per #175's pinned XED evidence
; (`PATTERN : 0x8D MOD[mm] MOD!=3 REG[rrr] RM[nnn] MODRM() REMOVE_SEGMENT()`,
; `OPERANDS : REG0=GPRv_R():w AGEN:r`). AGEN means the operand is an address
; computed from the ModRM/SIB/displacement, never an actual memory read --
; the identical addressing shape #199's mov-r64-mem-disp8 already encodes,
; only the opcode byte differs and no memory access happens. This is the
; first form the encoder admits that computes an address without touching
; any flags, needed once pointer/offset arithmetic (e.g. advancing a
; bump-pointer arena, or computing a struct field's address) must not
; disturb a CMP result still pending in a nearby branch.
(def x86-encode-lea-r64-mem-disp8
  (lambda (destination base displacement)
    (let ((dst (x86-reg-code destination)))
      (let ((base-code (x86-reg-code base)))
        (let ((rex (x86-encode-rex 1 (x86-high1 dst) 0 (x86-high1 base-code))))
          (let ((modrm (x86-encode-modrm 1 (x86-low3 dst) (x86-low3 base-code))))
            (cond
              ((eq (x86-low3 base-code) 4)
                (list
                  rex
                  141
                  modrm
                  (x86-encode-sib 0 4 4)
                  (x86-disp8-byte displacement)))
              (t
                (list rex 141 modrm (x86-disp8-byte displacement))))))))))

; Group-1 ALU r/m64, r64 (mod=3 register/register), opcode base+1: ADD 0x01,
; OR 0x09, AND 0x21, SUB 0x29, XOR 0x31, CMP 0x39 (Intel SDM, confirmed
; against #175's pinned XED evidence: lib/machine/xed/vendor/base/xed-isa.txt
; PATTERN lines for each ICLASS's `MOD[0b11] MOD=3 REG[rrr] RM[nnn]` form).
; All six share one shape; only the opcode byte differs.
(def x86-encode-alu-r64-r64
  (lambda (opcode destination source)
    (let ((dst (x86-reg-code destination)))
      (let ((src (x86-reg-code source)))
        (list
          (x86-encode-rex 1 (x86-high1 src) 0 (x86-high1 dst))
          opcode
          (x86-encode-modrm 3 (x86-low3 src) (x86-low3 dst)))))))

(def x86-encode-add-r64-r64
  (lambda (destination source)
    (x86-encode-alu-r64-r64 1 destination source)))

(def x86-encode-or-r64-r64
  (lambda (destination source)
    (x86-encode-alu-r64-r64 9 destination source)))

(def x86-encode-and-r64-r64
  (lambda (destination source)
    (x86-encode-alu-r64-r64 33 destination source)))

(def x86-encode-sub-r64-r64
  (lambda (destination source)
    (x86-encode-alu-r64-r64 41 destination source)))

(def x86-encode-xor-r64-r64
  (lambda (destination source)
    (x86-encode-alu-r64-r64 49 destination source)))

(def x86-encode-cmp-r64-r64
  (lambda (destination source)
    (x86-encode-alu-r64-r64 57 destination source)))

; MOV r/m64, r64 (opcode 0x89 /r, mod=3 register/register): copies source
; into destination with no flags touched -- per #175's pinned XED evidence
; (`PATTERN : 0x89 MOD[0b11] MOD=3 REG[rrr] RM[nnn]`, `OPERANDS : REG0=
; GPRv_B():w REG1=GPRv_R():r`, i.e. the reg field is the source being read
; and the rm field is the destination being written, the identical
; source/destination assignment the group-1 ALU family already uses). Same
; REX.W+opcode+ModRM shape, so it reuses x86-encode-alu-r64-r64 directly.
; This is the most elementary data-movement form still missing until now:
; every other admitted form can only load an immediate or a memory operand
; into a register, never copy register-to-register.
(def x86-encode-mov-r64-r64
  (lambda (destination source)
    (x86-encode-alu-r64-r64 137 destination source)))

; Two's-complement little-endian bytes for a sign-extended imm32 value
; already known to be in [-2147483648,2147483647]. Same wrap-before-split
; discipline #199 proved for disp8 (x86-disp8-byte) -- `mod` in this Lisp
; does not wrap negative operands, so adding 2^32 before reducing mod 2^32
; is exact for the whole imm32 domain.
(def x86-imm32-bytes
  (lambda (immediate)
    (x86-u32-bytes (mod (+ immediate 4294967296) 4294967296))))

; ALU r/m64, imm32 (mod=3, sign-extended to 64 bits): opcode 0x81, ModRM reg
; field selects the operation (ADD=0, OR=1, AND=4, SUB=5, XOR=6, CMP=7,
; matching the same fixed group-1 numbering the register/register family's
; own opcodes already encode), rm field is the destination register -- per
; #175's pinned XED evidence (`PATTERN : 0x81 MOD[0b11] MOD=3 REG[rrr]
; RM[nnn] SIMMz()`). Lets a comparison/arithmetic-against-a-constant (the
; #196 COND base-case shape: compare a variable to a literal) skip loading
; the constant into a scratch register first.
(def x86-encode-alu-r64-imm32
  (lambda (opcode-extension destination immediate)
    (let ((dst (x86-reg-code destination)))
      (cons
        (x86-encode-rex 1 0 0 (x86-high1 dst))
        (cons
          129
          (cons
            (x86-encode-modrm 3 opcode-extension (x86-low3 dst))
            (x86-imm32-bytes immediate)))))))

(def x86-encode-add-r64-imm32
  (lambda (destination immediate)
    (x86-encode-alu-r64-imm32 0 destination immediate)))

(def x86-encode-or-r64-imm32
  (lambda (destination immediate)
    (x86-encode-alu-r64-imm32 1 destination immediate)))

(def x86-encode-and-r64-imm32
  (lambda (destination immediate)
    (x86-encode-alu-r64-imm32 4 destination immediate)))

(def x86-encode-sub-r64-imm32
  (lambda (destination immediate)
    (x86-encode-alu-r64-imm32 5 destination immediate)))

(def x86-encode-xor-r64-imm32
  (lambda (destination immediate)
    (x86-encode-alu-r64-imm32 6 destination immediate)))

(def x86-encode-cmp-r64-imm32
  (lambda (destination immediate)
    (x86-encode-alu-r64-imm32 7 destination immediate)))

; TEST r/m64, r64 (opcode 0x85 /r, mod=3 register/register): destination AND
; source, result discarded, flags set only -- per #175's pinned XED evidence
; (`PATTERN : 0x85 MOD[0b11] MOD=3 REG[rrr] RM[nnn]`). Same REX.W+opcode+
; ModRM shape used by the group-1 ALU family, so it reuses x86-encode-alu-r64-r64
; directly rather than duplicating the REX/ModRM arithmetic.
(def x86-encode-test-r64-r64
  (lambda (destination source)
    (x86-encode-alu-r64-r64 133 destination source)))

; PUSH r64 (opcode 0x50+rd, ICLASS PUSH: `0b0101_0 SRM[rrr] ... DF64()`) and
; POP r64 (opcode 0x58+rd, ICLASS POP: `0b0101_1 SRM[rrr] ... DF64()`), per
; #175's pinned XED evidence. Both default to 64-bit operand size in long
; mode (`DF64()`), so no REX.W is emitted; only REX.B is needed, and only
; for r8-r15.
(def x86-encode-push-r64
  (lambda (register)
    (let ((code (x86-reg-code register)))
      (cond
        ((eq (x86-high1 code) 1)
          (list (x86-encode-rex 0 0 0 1) (+ 80 (x86-low3 code))))
        (t
          (list (+ 80 (x86-low3 code))))))))

(def x86-encode-pop-r64
  (lambda (register)
    (let ((code (x86-reg-code register)))
      (cond
        ((eq (x86-high1 code) 1)
          (list (x86-encode-rex 0 0 0 1) (+ 88 (x86-low3 code))))
        (t
          (list (+ 88 (x86-low3 code))))))))

; INC r64 / DEC r64: group-5 opcode 0xFF, /reg extension (not a register
; operand) selects the operation -- INC is /0, DEC is /1 -- per #175's
; pinned XED evidence (`PATTERN : 0xFF MOD[0b11] MOD=3 REG[0b000] RM[nnn]`
; / `REG[0b001]`). Unlike PUSH/POP, this form always needs REX.W: the
; legacy single-byte 0x40+r/0x48+r INC/DEC opcodes exist in the same pinned
; evidence tagged `not64` -- those byte values became REX prefixes in
; 64-bit mode, so encoding INC/DEC in long mode always goes through this
; ModRM group-5 path, never the legacy one.
(def x86-encode-inc-r64
  (lambda (register)
    (let ((code (x86-reg-code register)))
      (list
        (x86-encode-rex 1 0 0 (x86-high1 code))
        255
        (x86-encode-modrm 3 0 (x86-low3 code))))))

(def x86-encode-dec-r64
  (lambda (register)
    (let ((code (x86-reg-code register)))
      (list
        (x86-encode-rex 1 0 0 (x86-high1 code))
        255
        (x86-encode-modrm 3 1 (x86-low3 code))))))

; NOT r64 / NEG r64: group-3 opcode 0xF7, /reg extension (not a register
; operand) selects the operation -- NOT is /2, NEG is /3 -- per #175's
; pinned XED evidence (`PATTERN : 0xF7 MOD[0b11] MOD=3 REG[0b010] RM[nnn]`
; / `REG[0b011]`). This matches INC/DEC's group-5 shape: always REX.W,
; REX.B only for r8-r15.
(def x86-encode-not-r64
  (lambda (register)
    (let ((code (x86-reg-code register)))
      (list
        (x86-encode-rex 1 0 0 (x86-high1 code))
        247
        (x86-encode-modrm 3 2 (x86-low3 code))))))

(def x86-encode-neg-r64
  (lambda (register)
    (let ((code (x86-reg-code register)))
      (list
        (x86-encode-rex 1 0 0 (x86-high1 code))
        247
        (x86-encode-modrm 3 3 (x86-low3 code))))))

; SHL/SHR/SAR r64, imm8: group-2 opcode 0xC1, /reg extension selects the
; operation -- SHL is /4, SHR is /5, SAR is /7 (/6 duplicates SHL under an
; undocumented encoding and is intentionally not admitted) -- per #175's pinned XED
; evidence (`PATTERN : 0xC1 MOD[0b11] MOD=3 REG[0b100] RM[nnn] UIMM8()`
; for SHL, `REG[0b101]` for SHR, `REG[0b111]` for SAR). Unlike disp8/
; rel8/rel32/imm32, UIMM8 is an *unsigned* byte in [0,255], so no
; two's-complement wrap-before-split is needed -- the admitted count
; becomes the raw trailing byte directly.
(def x86-encode-shift-r64-imm8
  (lambda (opcode-extension register count)
    (let ((code (x86-reg-code register)))
      (list
        (x86-encode-rex 1 0 0 (x86-high1 code))
        193
        (x86-encode-modrm 3 opcode-extension (x86-low3 code))
        count))))

(def x86-encode-shl-r64-imm8
  (lambda (register count)
    (x86-encode-shift-r64-imm8 4 register count)))

(def x86-encode-shr-r64-imm8
  (lambda (register count)
    (x86-encode-shift-r64-imm8 5 register count)))

(def x86-encode-sar-r64-imm8
  (lambda (register count)
    (x86-encode-shift-r64-imm8 7 register count)))

; Jcc rel8: opcode 0x70+cc followed by a signed 8-bit relative displacement
; (from the address of the *next* instruction). No REX prefix -- this is a
; control-transfer, not a GPR operation. The 16 condition codes and their
; opcode offsets are fixed by Intel's encoding and confirmed against #175's
; pinned XED evidence (`PATTERN : 0x7<cc> mode64 ... BRDISP8()` for each
; ICLASS in order: JO,JNO,JB,JNB,JZ,JNZ,JBE,JNBE,JS,JNS,JP,JNP,JL,JNL,JLE,
; JNLE). Reuses x86-disp8-byte for the same two's-complement byte
; conversion #199 already proved correct for MOV's disp8 slot.
(def x86-encode-jcc-rel8
  (lambda (condition-code displacement)
    (list
      (+ 112 condition-code)
      (x86-disp8-byte displacement))))

(def x86-encode-jo-rel8 (lambda (displacement) (x86-encode-jcc-rel8 0 displacement)))
(def x86-encode-jno-rel8 (lambda (displacement) (x86-encode-jcc-rel8 1 displacement)))
(def x86-encode-jb-rel8 (lambda (displacement) (x86-encode-jcc-rel8 2 displacement)))
(def x86-encode-jnb-rel8 (lambda (displacement) (x86-encode-jcc-rel8 3 displacement)))
(def x86-encode-jz-rel8 (lambda (displacement) (x86-encode-jcc-rel8 4 displacement)))
(def x86-encode-jnz-rel8 (lambda (displacement) (x86-encode-jcc-rel8 5 displacement)))
(def x86-encode-jbe-rel8 (lambda (displacement) (x86-encode-jcc-rel8 6 displacement)))
(def x86-encode-jnbe-rel8 (lambda (displacement) (x86-encode-jcc-rel8 7 displacement)))
(def x86-encode-js-rel8 (lambda (displacement) (x86-encode-jcc-rel8 8 displacement)))
(def x86-encode-jns-rel8 (lambda (displacement) (x86-encode-jcc-rel8 9 displacement)))
(def x86-encode-jp-rel8 (lambda (displacement) (x86-encode-jcc-rel8 10 displacement)))
(def x86-encode-jnp-rel8 (lambda (displacement) (x86-encode-jcc-rel8 11 displacement)))
(def x86-encode-jl-rel8 (lambda (displacement) (x86-encode-jcc-rel8 12 displacement)))
(def x86-encode-jnl-rel8 (lambda (displacement) (x86-encode-jcc-rel8 13 displacement)))
(def x86-encode-jle-rel8 (lambda (displacement) (x86-encode-jcc-rel8 14 displacement)))
(def x86-encode-jnle-rel8 (lambda (displacement) (x86-encode-jcc-rel8 15 displacement)))

; JMP rel8: opcode 0xEB followed by a signed 8-bit relative displacement,
; confirmed against #175's pinned XED evidence
; (`PATTERN : 0xEB mode64 norex2_prefix FORCE64() BRDISP8()`). Unlike Jcc,
; JMP is unconditional -- no condition-code byte, no ModRM, no REX -- but
; reuses the same x86-disp8-byte two's-complement conversion.
(def x86-encode-jmp-rel8
  (lambda (displacement)
    (list
      235
      (x86-disp8-byte displacement))))

; Two's-complement little-endian bytes for a rel32 value already known to be
; in [-2147483648,2147483647]. Same wrap-before-split fix #199 proved for
; disp8 (x86-disp8-byte) and this file's own x86-u64-bytes lack for
; mov-r64-imm64 (#220's truth-sentinel audit documented this being an
; existing, out-of-scope gap) -- `mod` in this Lisp does not wrap negative operands, so
; adding 2^32 before reducing mod 2^32 is exact for the whole rel32 domain.
(def x86-rel32-bytes
  (lambda (displacement)
    (x86-u32-bytes (mod (+ displacement 4294967296) 4294967296))))

; CALL rel32: opcode 0xE8 followed by a signed 32-bit relative displacement
; (from the address of the *next* instruction), confirmed against #175's
; pinned XED evidence (`PATTERN : 0xE8 mode64 norex2_prefix BRDISP32()
; DF64() FORCE64()`). No REX, no ModRM -- like JMP rel8/Jcc rel8, this is a
; control-transfer, not a GPR operation. Unlike JMP, CALL also pushes the
; return address (XED_REG_STACKPUSH) -- on real hardware this is the actual
; RSP-based machine stack, so a CALL executed through native-call-u64-raw
; pushes/pops for real; nothing in the arena model needs to know about it,
; the same "guest ABI is Lisp's, host mechanism is the CPU's" split #204
; already established for the Windows/Linux execution adapters.
(def x86-encode-call-rel32
  (lambda (displacement)
    (cons 232 (x86-rel32-bytes displacement))))

; Jcc rel32: opcode 0x0F, then 0x80+cc, then a signed 32-bit relative
; displacement -- the conditional counterpart to JMP rel32, needed for the
; same reason: #196's own growth-v0 witnesses lean on CMP+Jcc far more than
; unconditional JMP (an arena-bounds check is inherently a Jcc), and their
; hand-derived disp8 offsets (e.g. "JNZ +11", "JNZ +33") will not survive
; much further composition. Confirmed against #175's pinned XED evidence
; (`PATTERN : 0x0F 0x8<cc> mode64 norex2_prefix FORCE64() BRANCH_HINT()
; BRDISP32()` for each ICLASS in the identical order the rel8 family
; already uses: JO,JNO,JB,JNB,JZ,JNZ,JBE,JNBE,JS,JNS,JP,JNP,JL,JNL,JLE,
; JNLE) -- reuses that same condition-code-to-opcode-offset mapping, just
; with a 2-byte opcode and a 4-byte displacement instead of 1+1.
(def x86-encode-jcc-rel32
  (lambda (condition-code displacement)
    (cons
      15
      (cons
        (+ 128 condition-code)
        (x86-rel32-bytes displacement)))))

(def x86-encode-jo-rel32 (lambda (displacement) (x86-encode-jcc-rel32 0 displacement)))
(def x86-encode-jno-rel32 (lambda (displacement) (x86-encode-jcc-rel32 1 displacement)))
(def x86-encode-jb-rel32 (lambda (displacement) (x86-encode-jcc-rel32 2 displacement)))
(def x86-encode-jnb-rel32 (lambda (displacement) (x86-encode-jcc-rel32 3 displacement)))
(def x86-encode-jz-rel32 (lambda (displacement) (x86-encode-jcc-rel32 4 displacement)))
(def x86-encode-jnz-rel32 (lambda (displacement) (x86-encode-jcc-rel32 5 displacement)))
(def x86-encode-jbe-rel32 (lambda (displacement) (x86-encode-jcc-rel32 6 displacement)))
(def x86-encode-jnbe-rel32 (lambda (displacement) (x86-encode-jcc-rel32 7 displacement)))
(def x86-encode-js-rel32 (lambda (displacement) (x86-encode-jcc-rel32 8 displacement)))
(def x86-encode-jns-rel32 (lambda (displacement) (x86-encode-jcc-rel32 9 displacement)))
(def x86-encode-jp-rel32 (lambda (displacement) (x86-encode-jcc-rel32 10 displacement)))
(def x86-encode-jnp-rel32 (lambda (displacement) (x86-encode-jcc-rel32 11 displacement)))
(def x86-encode-jl-rel32 (lambda (displacement) (x86-encode-jcc-rel32 12 displacement)))
(def x86-encode-jnl-rel32 (lambda (displacement) (x86-encode-jcc-rel32 13 displacement)))
(def x86-encode-jle-rel32 (lambda (displacement) (x86-encode-jcc-rel32 14 displacement)))
(def x86-encode-jnle-rel32 (lambda (displacement) (x86-encode-jcc-rel32 15 displacement)))

; JMP rel32: opcode 0xE9 followed by a signed 32-bit relative displacement,
; confirmed against #175's pinned XED evidence (`PATTERN : 0xE9 mode64
; norex2_prefix FORCE64() BRDISP32()`). Matches JMP rel8's own
; unconditional, no-ModRM, no-REX shape -- only the opcode byte and
; displacement width differ.
(def x86-encode-jmp-rel32
  (lambda (displacement)
    (cons 233 (x86-rel32-bytes displacement))))

; CALL r64 / JMP r64 (indirect through a register): group-5 opcode 0xFF,
; /reg extension selects the operation -- CALL is /2, JMP is /4 -- per
; #175's pinned XED evidence (`PATTERN : 0xFF MOD[0b11] MOD=3 REG[0b010]
; RM[nnn] DF64() ...` for CALL, `REG[0b100]` for JMP). Both are `DF64()`
; (default 64-bit operand size in long mode), matching PUSH/POP's own
; default-64-bit shape, so no REX.W is emitted -- only REX.B, and only for
; r8-r15. Unlike CALL rel32,
; the target here is whatever absolute address the admitted register holds
; at runtime, not a displacement fixed at encode time; CALL still pushes a
; real return address via XED_REG_STACKPUSH, onto the same real machine
; stack #204 already established. This is the first indirect (register-
; target) control transfer the encoder admits: the destination is
; genuinely a runtime value, not something knowable from the bytes alone.
(def x86-encode-group5-indirect-r64
  (lambda (opcode-extension register)
    (let ((code (x86-reg-code register)))
      (cond
        ((eq (x86-high1 code) 1)
          (list
            (x86-encode-rex 0 0 0 1)
            255
            (x86-encode-modrm 3 opcode-extension (x86-low3 code))))
        (t
          (list
            255
            (x86-encode-modrm 3 opcode-extension (x86-low3 code))))))))

(def x86-encode-call-r64
  (lambda (register)
    (x86-encode-group5-indirect-r64 2 register)))

(def x86-encode-jmp-r64
  (lambda (register)
    (x86-encode-group5-indirect-r64 4 register)))

(def x86-encode-program
  (lambda (instructions)
    (cond
      ((atom instructions) (quote ()))
      (t
        (append
          (car instructions)
          (x86-encode-program (cdr instructions)))))))
