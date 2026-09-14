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
                  (mod displacement 256)))
              (t
                (list rex 139 modrm (mod displacement 256))))))))))

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
                  (mod displacement 256)))
              (t
                (list rex 137 modrm (mod displacement 256))))))))))

(def x86-encode-add-r64-r64
  (lambda (destination source)
    (let ((dst (x86-reg-code destination)))
      (let ((src (x86-reg-code source)))
        (list
          (x86-encode-rex 1 (x86-high1 src) 0 (x86-high1 dst))
          1
          (x86-encode-modrm 3 (x86-low3 src) (x86-low3 dst)))))))

(def x86-encode-program
  (lambda (instructions)
    (cond
      ((atom instructions) (quote ()))
      (t
        (append
          (car instructions)
          (x86-encode-program (cdr instructions)))))))
