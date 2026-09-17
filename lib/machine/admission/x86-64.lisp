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
     (mov-r64-imm64 register immediate)
     (add-r64-r64 register register)
     (or-r64-r64 register register)
     (and-r64-r64 register register)
     (sub-r64-r64 register register)
     (xor-r64-r64 register register)
     (cmp-r64-r64 register register)
     (test-r64-r64 register register)
     (push-r64 register)
     (pop-r64 register)
     (inc-r64 register)
     (dec-r64 register)
     (not-r64 register)
     (neg-r64 register)
     (mov-mem-disp8-r64 register disp8 register)
     (mov-r64-mem-disp8 register register disp8)
     (jo-rel8 disp8)
     (jno-rel8 disp8)
     (jb-rel8 disp8)
     (jnb-rel8 disp8)
     (jz-rel8 disp8)
     (jnz-rel8 disp8)
     (jbe-rel8 disp8)
     (jnbe-rel8 disp8)
     (js-rel8 disp8)
     (jns-rel8 disp8)
     (jp-rel8 disp8)
     (jnp-rel8 disp8)
     (jl-rel8 disp8)
     (jnl-rel8 disp8)
     (jle-rel8 disp8)
     (jnle-rel8 disp8)
     (jmp-rel8 disp8)
     (jmp-rel32 rel32))))

; A disp8 slot only admits an exact integer in [-128,127]. Comparison
; operators like `>=` error on a non-number rather than returning () (a
; naive `register`-style guard would crash admission on a symbol/list
; operand instead of failing closed), so this checks the operand's own
; printed form character-by-character first -- the same digit-classifying
; approach #177's typed operands layer uses, kept self-contained here since
; admission must not depend on loading that file.
(def x86-admission-decimal-digit?
  (lambda (character)
    (member? character
             (quote ("0" "1" "2" "3" "4" "5" "6" "7" "8" "9")))))

(def x86-admission-decimal-digits?
  (lambda (text)
    (cond
      ((string-empty? text) t)
      ((x86-admission-decimal-digit? (string-first text))
       (x86-admission-decimal-digits? (string-rest text)))
      (t (quote ())))))

(def x86-admission-exact-integer?
  (lambda (value)
    (cond
      ((symbol? value) (quote ()))
      ((not (atom value)) (quote ()))
      (t
       (let ((text (write-to-string value)))
         (cond
           ((string-empty? text) (quote ()))
           ((eq (string-first text) "-")
            (cond
              ((string-empty? (string-rest text)) (quote ()))
              (t (x86-admission-decimal-digits? (string-rest text)))))
           (t (x86-admission-decimal-digits? text))))))))

(def x86-admission-disp8?
  (lambda (value)
    (cond
      ((x86-admission-exact-integer? value)
       (and (>= value -128) (<= value 127)))
      (t (quote ())))))

; A rel32 slot only admits an exact integer in the signed 32-bit range.
(def x86-admission-rel32?
  (lambda (value)
    (cond
      ((x86-admission-exact-integer? value)
       (and (>= value -2147483648) (<= value 2147483647)))
      (t (quote ())))))

; `immediate`, `register`, and `disp8` are operand-slot wildcards, not
; opcode wildcards. `register` only admits the 16 GPR names x86-reg-code
; knows about -- any other atom (a number, a made-up symbol) fails the
; match, the same way an out-of-range immediate would fail the encoder
; later. The selected Lisp encoder still validates whether the operand can
; be represented before the raw host capability is reachable.
(def x86-admission-pattern-match?
  (lambda (pattern form)
    (cond
      ((atom pattern)
       (cond
         ((eq pattern (quote immediate)) t)
         ((eq pattern (quote register))
          (cond
            ((atom form) (not (eq (x86-reg-code form) (quote ()))))
            (t (quote ()))))
         ((eq pattern (quote disp8))
          (cond
            ((atom form) (x86-admission-disp8? form))
            (t (quote ()))))
         ((eq pattern (quote rel32))
          (cond
            ((atom form) (x86-admission-rel32? form))
            (t (quote ()))))
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
      ((x86-admission-pattern-match? (quote (mov-r64-imm64 register immediate)) form)
       (x86-encode-mov-r64-imm64 (second form) (third form)))
      ((x86-admission-pattern-match? (quote (add-r64-r64 register register)) form)
       (x86-encode-add-r64-r64 (second form) (third form)))
      ((x86-admission-pattern-match? (quote (or-r64-r64 register register)) form)
       (x86-encode-or-r64-r64 (second form) (third form)))
      ((x86-admission-pattern-match? (quote (and-r64-r64 register register)) form)
       (x86-encode-and-r64-r64 (second form) (third form)))
      ((x86-admission-pattern-match? (quote (sub-r64-r64 register register)) form)
       (x86-encode-sub-r64-r64 (second form) (third form)))
      ((x86-admission-pattern-match? (quote (xor-r64-r64 register register)) form)
       (x86-encode-xor-r64-r64 (second form) (third form)))
      ((x86-admission-pattern-match? (quote (cmp-r64-r64 register register)) form)
       (x86-encode-cmp-r64-r64 (second form) (third form)))
      ((x86-admission-pattern-match? (quote (test-r64-r64 register register)) form)
       (x86-encode-test-r64-r64 (second form) (third form)))
      ((x86-admission-pattern-match? (quote (push-r64 register)) form)
       (x86-encode-push-r64 (second form)))
      ((x86-admission-pattern-match? (quote (pop-r64 register)) form)
       (x86-encode-pop-r64 (second form)))
      ((x86-admission-pattern-match? (quote (inc-r64 register)) form)
       (x86-encode-inc-r64 (second form)))
      ((x86-admission-pattern-match? (quote (dec-r64 register)) form)
       (x86-encode-dec-r64 (second form)))
      ((x86-admission-pattern-match? (quote (not-r64 register)) form)
       (x86-encode-not-r64 (second form)))
      ((x86-admission-pattern-match? (quote (neg-r64 register)) form)
       (x86-encode-neg-r64 (second form)))
      ((x86-admission-pattern-match? (quote (mov-mem-disp8-r64 register disp8 register)) form)
       (x86-encode-mov-mem-disp8-r64 (second form) (third form) (fourth form)))
      ((x86-admission-pattern-match? (quote (mov-r64-mem-disp8 register register disp8)) form)
       (x86-encode-mov-r64-mem-disp8 (second form) (third form) (fourth form)))
      ((x86-admission-pattern-match? (quote (jo-rel8 disp8)) form)
       (x86-encode-jo-rel8 (second form)))
      ((x86-admission-pattern-match? (quote (jno-rel8 disp8)) form)
       (x86-encode-jno-rel8 (second form)))
      ((x86-admission-pattern-match? (quote (jb-rel8 disp8)) form)
       (x86-encode-jb-rel8 (second form)))
      ((x86-admission-pattern-match? (quote (jnb-rel8 disp8)) form)
       (x86-encode-jnb-rel8 (second form)))
      ((x86-admission-pattern-match? (quote (jz-rel8 disp8)) form)
       (x86-encode-jz-rel8 (second form)))
      ((x86-admission-pattern-match? (quote (jnz-rel8 disp8)) form)
       (x86-encode-jnz-rel8 (second form)))
      ((x86-admission-pattern-match? (quote (jbe-rel8 disp8)) form)
       (x86-encode-jbe-rel8 (second form)))
      ((x86-admission-pattern-match? (quote (jnbe-rel8 disp8)) form)
       (x86-encode-jnbe-rel8 (second form)))
      ((x86-admission-pattern-match? (quote (js-rel8 disp8)) form)
       (x86-encode-js-rel8 (second form)))
      ((x86-admission-pattern-match? (quote (jns-rel8 disp8)) form)
       (x86-encode-jns-rel8 (second form)))
      ((x86-admission-pattern-match? (quote (jp-rel8 disp8)) form)
       (x86-encode-jp-rel8 (second form)))
      ((x86-admission-pattern-match? (quote (jnp-rel8 disp8)) form)
       (x86-encode-jnp-rel8 (second form)))
      ((x86-admission-pattern-match? (quote (jl-rel8 disp8)) form)
       (x86-encode-jl-rel8 (second form)))
      ((x86-admission-pattern-match? (quote (jnl-rel8 disp8)) form)
       (x86-encode-jnl-rel8 (second form)))
      ((x86-admission-pattern-match? (quote (jle-rel8 disp8)) form)
       (x86-encode-jle-rel8 (second form)))
      ((x86-admission-pattern-match? (quote (jnle-rel8 disp8)) form)
       (x86-encode-jnle-rel8 (second form)))
      ((x86-admission-pattern-match? (quote (jmp-rel8 disp8)) form)
       (x86-encode-jmp-rel8 (second form)))
      ((x86-admission-pattern-match? (quote (jmp-rel32 rel32)) form)
       (x86-encode-jmp-rel32 (second form)))
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
