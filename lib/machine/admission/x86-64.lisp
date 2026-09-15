; Закритий admission-шар для bounded x86-64 native witness.
; Closed admission layer for the bounded x86-64 native witness.
;
; Це НЕ x86 decoder і НЕ друга ISA-authority. Таблиця нижче описує лише
; форми, які my-lisp зараз допускає до canonical native gateway. Opcode/REX/
; ModRM facts залишаються в Lisp-owned encoder; hardware spec лишається ISA
; authority. Усе, що не збігається з таблицею, відхиляється до host.
;
; This is NOT an x86 decoder and NOT a second ISA authority. The table below
; names only the structured forms currently admitted to the canonical native
; gateway. Opcode/REX/ModRM facts stay in the Lisp-owned encoder; the hardware
; specification remains ISA authority. Everything else fails closed before host.

(def x86-admitted-instruction-patterns
  (quote
    ((ret)
     (mov-r64-imm64 rax immediate)
     (mov-r64-imm64 rcx immediate)
     (add-r64-r64 rax rcx)
     (mov-mem-disp8-r64 rdi 0 rax)
     (mov-mem-disp8-r64 rdi 8 rax)
     (mov-r64-mem-disp8 rax rdi 0)
     (mov-r64-mem-disp8 rax rdi 8))))

; `immediate` is an operand slot, not an opcode wildcard. The selected Lisp
; encoder still validates whether the operand can be represented before the
; raw host capability is reachable.
(def x86-admission-pattern-match?
  (lambda (pattern form)
    (cond
      ((atom pattern)
       (cond
         ((eq pattern (quote immediate)) t)
         ((atom form) (eq pattern form))
         (t (quote ()))))
      ((atom form) (quote ()))
      ((x86-admission-pattern-match? (car pattern) (car form))
       (x86-admission-pattern-match? (cdr pattern) (cdr form)))
      (t (quote ())))))

(def x86-admitted-instruction-against?
  (lambda (patterns form)
    (cond
      ((atom patterns) (quote ()))
      ((x86-admission-pattern-match? (car patterns) form) t)
      (t (x86-admitted-instruction-against? (cdr patterns) form)))))

(def x86-admitted-instruction?
  (lambda (form)
    (x86-admitted-instruction-against? x86-admitted-instruction-patterns form)))

(def x86-first-unadmitted-form
  (lambda (forms)
    (cond
      ((atom forms)
       (cond
         ((eq forms (quote ())) (quote ()))
         (t forms)))
      ((x86-admitted-instruction? (car forms))
       (x86-first-unadmitted-form (cdr forms)))
      (t (car forms)))))

(def x86-admitted-program?
  (lambda (forms)
    (cond
      ((atom forms) (eq forms (quote ())))
      ((x86-admitted-instruction? (car forms))
       (x86-admitted-program? (cdr forms)))
      (t (quote ())))))

(def x86-encode-admitted-instruction
  (lambda (form)
    (cond
      ((equal? form (quote (ret)))
       (x86-encode-ret))
      ((x86-admission-pattern-match? (quote (mov-r64-imm64 rax immediate)) form)
       (x86-encode-mov-r64-imm64 (quote rax) (third form)))
      ((x86-admission-pattern-match? (quote (mov-r64-imm64 rcx immediate)) form)
       (x86-encode-mov-r64-imm64 (quote rcx) (third form)))
      ((equal? form (quote (add-r64-r64 rax rcx)))
       (x86-encode-add-r64-r64 (quote rax) (quote rcx)))
      ((equal? form (quote (mov-mem-disp8-r64 rdi 0 rax)))
       (x86-encode-mov-mem-disp8-r64 (quote rdi) 0 (quote rax)))
      ((equal? form (quote (mov-mem-disp8-r64 rdi 8 rax)))
       (x86-encode-mov-mem-disp8-r64 (quote rdi) 8 (quote rax)))
      ((equal? form (quote (mov-r64-mem-disp8 rax rdi 0)))
       (x86-encode-mov-r64-mem-disp8 (quote rax) (quote rdi) 0))
      ((equal? form (quote (mov-r64-mem-disp8 rax rdi 8)))
       (x86-encode-mov-r64-mem-disp8 (quote rax) (quote rdi) 8))
      ; Unreachable after admission. Keep fail-closed data instead of inventing
      ; a fallback encoder.
      (t (quote ())))))

(def x86-encode-admitted-program
  (lambda (forms)
    (x86-encode-program (map x86-encode-admitted-instruction forms))))

; #177 safe composition seam. Unlike x86-encode-admitted-program, this entry
; point is safe for callers holding arbitrary structured machine data: it must
; prove admission before any byte materialization can happen.
(def x86-encode-admitted-program-or-reject
  (lambda (forms)
    (cond
      ((x86-admitted-program? forms)
       (x86-encode-admitted-program forms))
      (t
       (list (quote rejected)
             (quote unadmitted-machine-form)
             (x86-first-unadmitted-form forms))))))

(def x86-call-admitted-u64
  (lambda (forms arena-bytes)
    (cond
      ((x86-admitted-program? forms)
       (cond
         ((eq arena-bytes 0)
          (native-call-u64-raw (x86-encode-admitted-program forms)))
         (t
          (native-call-u64-raw (x86-encode-admitted-program forms) arena-bytes))))
      (t
       (list (quote rejected)
             (quote unadmitted-machine-form)
             (x86-first-unadmitted-form forms))))))
