; #177 — thin x86-64 machine atoms over the structured forms admitted by #176.
; These constructors own composition only. They intentionally contain no
; opcodes, byte strings, register-code tables, or CPU feature tables.
;
; Raw operands remain accepted for compatibility with the first #177 slice,
; but are normalized through the typed operand layer before a machine form is
; constructed. Operand typing does not grant instruction admission.

(def x86-ret
  (lambda ()
    (quote (ret))))

(def x86-mov-r64-imm64
  (lambda (register immediate)
    (let ((typed-register (x86-as-gpr64 register)))
      (cond
        ((x86-machine-rejected? typed-register) typed-register)
        (t
         (let ((typed-immediate (x86-as-u64-imm immediate)))
           (cond
             ((x86-machine-rejected? typed-immediate) typed-immediate)
             (t
              (list (quote mov-r64-imm64)
                    (x86-gpr64-value typed-register)
                    (x86-u64-imm-value typed-immediate))))))))))

(def x86-add-r64-r64
  (lambda (destination source)
    (let ((typed-destination (x86-as-gpr64 destination)))
      (cond
        ((x86-machine-rejected? typed-destination) typed-destination)
        (t
         (let ((typed-source (x86-as-gpr64 source)))
           (cond
             ((x86-machine-rejected? typed-source) typed-source)
             (t
              (list (quote add-r64-r64)
                    (x86-gpr64-value typed-destination)
                    (x86-gpr64-value typed-source))))))))))

(def x86-mov-mem64-r64
  (lambda (memory source)
    (let ((typed-memory (x86-as-mem64-disp8 memory)))
      (cond
        ((x86-machine-rejected? typed-memory) typed-memory)
        (t
         (let ((typed-source (x86-as-gpr64 source)))
           (cond
             ((x86-machine-rejected? typed-source) typed-source)
             (t
              (list (quote mov-mem-disp8-r64)
                    (x86-mem64-disp8-base typed-memory)
                    (x86-mem64-disp8-displacement typed-memory)
                    (x86-gpr64-value typed-source))))))))))

(def x86-mov-r64-mem64
  (lambda (destination memory)
    (let ((typed-destination (x86-as-gpr64 destination)))
      (cond
        ((x86-machine-rejected? typed-destination) typed-destination)
        (t
         (let ((typed-memory (x86-as-mem64-disp8 memory)))
           (cond
             ((x86-machine-rejected? typed-memory) typed-memory)
             (t
              (list (quote mov-r64-mem-disp8)
                    (x86-gpr64-value typed-destination)
                    (x86-mem64-disp8-base typed-memory)
                    (x86-mem64-disp8-displacement typed-memory))))))))))

; Compatibility constructors for the first #177 foundation. They now pass
; through typed memory/register validation and project back to the canonical
; #176-native forms.
(def x86-mov-mem-disp8-r64
  (lambda (base displacement source)
    (let ((memory (x86-mem64-disp8 base displacement)))
      (cond
        ((x86-machine-rejected? memory) memory)
        (t (x86-mov-mem64-r64 memory source))))))

(def x86-mov-r64-mem-disp8
  (lambda (destination base displacement)
    (let ((memory (x86-mem64-disp8 base displacement)))
      (cond
        ((x86-machine-rejected? memory) memory)
        (t (x86-mov-r64-mem64 destination memory))))))

(def x86-encode-machine-block
  (lambda (block)
    (x86-encode-admitted-program-or-reject (machine-block-forms block))))
