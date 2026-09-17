; #177 — typed x86-64 machine operands as Lisp-owned data.
;
; This layer owns operand *shape*, not instruction admission and not encoding.
; Register knowledge is intentionally reused from x86-reg-code in the existing
; Lisp encoder so this file cannot become a second GPR table. A well-typed
; operand may still participate in an instruction form that #176 admission
; rejects; type success is not capability/admission success.

(def x86-machine-operand-rejection
  (lambda (expected actual)
    (list (quote rejected) (quote machine-operand) expected actual)))

(def x86-machine-rejected?
  (lambda (value)
    (cond
      ((atom value) (quote ()))
      ((eq (car value) (quote rejected)) t)
      (t (quote ())))))

(def x86-gpr8-name?
  (lambda (name)
    (cond
      ((symbol? name)
       (member?
         name
         (quote
           (al cl dl bl spl bpl sil dil
               r8b r9b r10b r11b r12b r13b r14b r15b))))
      (t (quote ())))))

(def x86-gpr64-name?
  (lambda (name)
    (cond
      ((symbol? name)
       (and
         (not (eq (x86-reg-code name) (quote ())))
         (not (x86-gpr8-name? name))))
      (t (quote ())))))

(def x86-gpr8?
  (lambda (operand)
    (cond
      ((atom operand) (quote ()))
      ((eq (car operand) (quote gpr8))
       (cond
         ((atom (cdr operand)) (quote ()))
         ((equal? (cdr (cdr operand)) (quote ()))
          (x86-gpr8-name? (second operand)))
         (t (quote ()))))
      (t (quote ())))))

(def x86-gpr8
  (lambda (name)
    (cond
      ((x86-gpr8-name? name) (list (quote gpr8) name))
      (t (x86-machine-operand-rejection (quote gpr8) name)))))

(def x86-as-gpr8
  (lambda (operand)
    (cond
      ((x86-machine-rejected? operand) operand)
      ((x86-gpr8? operand) operand)
      (t (x86-gpr8 operand)))))

(def x86-gpr8-value
  (lambda (operand)
    (second operand)))

(def x86-gpr64?
  (lambda (operand)
    (cond
      ((atom operand) (quote ()))
      ((eq (car operand) (quote gpr64))
       (cond
         ((atom (cdr operand)) (quote ()))
         ((equal? (cdr (cdr operand)) (quote ()))
          (x86-gpr64-name? (second operand)))
         (t (quote ()))))
      (t (quote ())))))

(def x86-gpr64
  (lambda (name)
    (cond
      ((x86-gpr64-name? name) (list (quote gpr64) name))
      (t (x86-machine-operand-rejection (quote gpr64) name)))))

(def x86-as-gpr64
  (lambda (operand)
    (cond
      ((x86-machine-rejected? operand) operand)
      ((x86-gpr64? operand) operand)
      (t (x86-gpr64 operand)))))

(def x86-gpr64-value
  (lambda (operand)
    (second operand)))

; Exact integer recognition stays in Lisp and does not require a new host
; `number?`/`integer?` primitive. Canonical serialization is already a language
; contract: exact integers print as optional '-' followed only by decimal
; digits; exact non-integral rationals contain '/', and inexact values retain a
; decimal marker. symbol? is checked first so a symbol created with numeric text
; cannot impersonate a number.
(def x86-decimal-digit?
  (lambda (character)
    (member? character
             (quote ("0" "1" "2" "3" "4" "5" "6" "7" "8" "9")))))

(def x86-decimal-digits?
  (lambda (text)
    (cond
      ((string-empty? text) t)
      ((x86-decimal-digit? (string-first text))
       (x86-decimal-digits? (string-rest text)))
      (t (quote ())))))

(def x86-exact-integer?
  (lambda (value)
    (cond
      ((not (atom value)) (quote ()))
      ((symbol? value) (quote ()))
      (t
       (let ((text (write-to-string value)))
         (cond
           ((string-empty? text) (quote ()))
           ((eq (string-first text) "-")
            (cond
              ((string-empty? (string-rest text)) (quote ()))
              (t (x86-decimal-digits? (string-rest text)))))
           (t (x86-decimal-digits? text))))))))

(def x86-u64-imm?
  (lambda (operand)
    (cond
      ((atom operand) (quote ()))
      ((eq (car operand) (quote u64-imm))
       (cond
         ((atom (cdr operand)) (quote ()))
         ((equal? (cdr (cdr operand)) (quote ()))
          (let ((value (second operand)))
            (cond
              ((x86-exact-integer? value)
               (and (>= value 0) (<= value 18446744073709551615)))
              (t (quote ())))))
         (t (quote ()))))
      (t (quote ())))))

(def x86-u64-imm
  (lambda (value)
    (cond
      ((x86-exact-integer? value)
       (cond
         ((and (>= value 0) (<= value 18446744073709551615))
          (list (quote u64-imm) value))
         (t (x86-machine-operand-rejection (quote u64-imm) value))))
      (t (x86-machine-operand-rejection (quote u64-imm) value)))))

(def x86-as-u64-imm
  (lambda (operand)
    (cond
      ((x86-machine-rejected? operand) operand)
      ((x86-u64-imm? operand) operand)
      (t (x86-u64-imm operand)))))

(def x86-u64-imm-value
  (lambda (operand)
    (second operand)))

(def x86-disp8?
  (lambda (operand)
    (cond
      ((atom operand) (quote ()))
      ((eq (car operand) (quote disp8))
       (cond
         ((atom (cdr operand)) (quote ()))
         ((equal? (cdr (cdr operand)) (quote ()))
          (let ((value (second operand)))
            (cond
              ((x86-exact-integer? value)
               (and (>= value -128) (<= value 127)))
              (t (quote ())))))
         (t (quote ()))))
      (t (quote ())))))

(def x86-disp8
  (lambda (value)
    (cond
      ((x86-exact-integer? value)
       (cond
         ((and (>= value -128) (<= value 127))
          (list (quote disp8) value))
         (t (x86-machine-operand-rejection (quote disp8) value))))
      (t (x86-machine-operand-rejection (quote disp8) value)))))

(def x86-as-disp8
  (lambda (operand)
    (cond
      ((x86-machine-rejected? operand) operand)
      ((x86-disp8? operand) operand)
      (t (x86-disp8 operand)))))

(def x86-disp8-value
  (lambda (operand)
    (second operand)))

(def x86-mem64-disp8?
  (lambda (operand)
    (cond
      ((atom operand) (quote ()))
      ((eq (car operand) (quote mem64-disp8))
       (cond
         ((atom (cdr operand)) (quote ()))
         ((atom (cdr (cdr operand))) (quote ()))
         ((equal? (cdr (cdr (cdr operand))) (quote ()))
          (and (x86-gpr64? (second operand))
               (x86-disp8? (third operand))))
         (t (quote ()))))
      (t (quote ())))))

(def x86-mem64-disp8
  (lambda (base displacement)
    (let ((typed-base (x86-as-gpr64 base)))
      (cond
        ((x86-machine-rejected? typed-base) typed-base)
        (t
         (let ((typed-displacement (x86-as-disp8 displacement)))
           (cond
             ((x86-machine-rejected? typed-displacement) typed-displacement)
             (t
              (list (quote mem64-disp8)
                    typed-base
                    typed-displacement)))))))))

(def x86-as-mem64-disp8
  (lambda (operand)
    (cond
      ((x86-machine-rejected? operand) operand)
      ((x86-mem64-disp8? operand) operand)
      (t (x86-machine-operand-rejection (quote mem64-disp8) operand)))))

(def x86-mem64-disp8-base
  (lambda (operand)
    (x86-gpr64-value (second operand))))

(def x86-mem64-disp8-displacement
  (lambda (operand)
    (x86-disp8-value (third operand))))
