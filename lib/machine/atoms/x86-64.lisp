; #177 — thin x86-64 machine atoms over the structured forms admitted by #176.
; These constructors own composition only. They intentionally contain no
; opcodes, byte strings, register-code tables, or CPU feature tables.

(def x86-ret
  (lambda ()
    (quote (ret))))

(def x86-mov-r64-imm64
  (lambda (register immediate)
    (list (quote mov-r64-imm64) register immediate)))

(def x86-add-r64-r64
  (lambda (destination source)
    (list (quote add-r64-r64) destination source)))

(def x86-mov-mem-disp8-r64
  (lambda (base displacement source)
    (list (quote mov-mem-disp8-r64) base displacement source)))

(def x86-mov-r64-mem-disp8
  (lambda (destination base displacement)
    (list (quote mov-r64-mem-disp8) destination base displacement)))

(def x86-encode-machine-block
  (lambda (block)
    (x86-encode-admitted-program-or-reject (machine-block-forms block))))
