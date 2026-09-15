; #177 — architecture-neutral machine block composition.
; A machine block is deliberately only an ordered list of structured machine
; forms. It owns no ISA facts, opcodes, bytes, feature tables, or semantics.
; Target-specific atoms create the forms; admission decides whether a target
; accepts them; the encoder materializes bytes only after admission.

(def machine-block
  (lambda (forms)
    forms))

(def machine-block-empty
  (lambda ()
    (quote ())))

(def machine-block-one
  (lambda (form)
    (list form)))

(def machine-block-append
  (lambda (block form)
    (append block (list form))))

(def machine-block-concat
  (lambda (left right)
    (append left right)))

(def machine-block-forms
  (lambda (block)
    block))
