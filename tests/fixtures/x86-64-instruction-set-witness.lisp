; #176 — x86-64 instruction harvest witness (LEA, JMP rel32, Jcc rel32).
; Admitted machine forms must encode deterministically into exact machine
; bytes and validate their bounds at the closed admission boundary.
; The host/shell observes only the named pass envelope below.

(load "lib/core.lisp")
(load "lib/machine/encoding/x86-64.lisp")
(load "lib/machine/admission/x86-64.lisp")

(def x86-instruction-witness-tests
  (quote
    (((encode lea-basic)
      (x86-encode-lea-r64-mem-disp8 (quote rax) (quote rbx) 10)
      (72 141 67 10))
     ((encode lea-rsp-sib)
      (x86-encode-lea-r64-mem-disp8 (quote rax) (quote rsp) 8)
      (72 141 68 36 8))
     ((encode lea-r8-rex)
      (x86-encode-lea-r64-mem-disp8 (quote rax) (quote r8) 12)
      (73 141 64 12))
     ((encode jmp-rel32-pos)
      (x86-encode-jmp-rel32 200)
      (233 200 0 0 0))
     ((encode jmp-rel32-neg)
      (x86-encode-jmp-rel32 -100)
      (233 156 255 255 255))
     ((encode jo-rel32)
      (x86-encode-jo-rel32 500)
      (15 128 244 1 0 0))
     ((encode jz-rel32)
      (x86-encode-jz-rel32 1000)
      (15 132 232 3 0 0))
     ((encode jnz-rel32)
      (x86-encode-jnz-rel32 -50)
      (15 133 206 255 255 255))
     ((encode jnle-rel32)
      (x86-encode-jnle-rel32 16)
      (15 143 16 0 0 0))
     ((admission lea-valid)
      (x86-admitted-instruction? (quote (lea-r64-mem-disp8 rax rbx 10)))
      t)
     ((admission lea-overflow)
      (x86-admitted-instruction? (quote (lea-r64-mem-disp8 rax rbx 200)))
      ())
     ((admission jmp-rel32-valid)
      (x86-admitted-instruction? (quote (jmp-rel32 50000)))
      t)
     ((admission jmp-rel32-overflow)
      (x86-admitted-instruction? (quote (jmp-rel32 3000000000)))
      ())
     ((admission jz-rel32-valid)
      (x86-admitted-instruction? (quote (jz-rel32 -100000)))
      t)
     ((admission jnz-rel32-overflow)
      (x86-admitted-instruction? (quote (jnz-rel32 -3000000000)))
      ())
     ((encode program)
      (x86-encode-admitted-program
        (quote
          ((lea-r64-mem-disp8 rax rbx 10)
           (jz-rel32 1000)
           (jmp-rel32 -100))))
      (72 141 67 10 15 132 232 3 0 0 233 156 255 255 255)))))

(def x86-instruction-eval-row
  (lambda (row)
    (let ((name (car row))
          (expr (second row))
          (expected (third row)))
      (let ((actual (eval expr)))
        (cond
          ((equal? actual expected) (structural-relation same)
           (quote pass))
          (t
           (list (quote fail) name expected actual)))))))

(def x86-instruction-run-all
  (lambda (rows)
    (cond
      ((atom rows) (quote (x86-64-instruction-set-witness (status pass))))
      (t
       (let ((result (x86-instruction-eval-row (car rows))))
         (cond
           ((eq result (quote pass))
            (x86-instruction-run-all (cdr rows)))
           (t
            (list
              (quote x86-64-instruction-set-witness)
              (quote (status fail))
              (list (quote check) (second result))
              (list (quote expected) (third result))
              (list (quote actual) (fourth result))))))))))

(x86-instruction-run-all x86-instruction-witness-tests)
