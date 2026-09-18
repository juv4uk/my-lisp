; Experimental CPU bootstrap witness for () -> 00000000.
; This is laboratory code only. It reuses the existing Lisp-owned x86 machine
; atoms/admission/encoder and the semantics-blind native executor.
;
; The CPU is allowed to materialize bits. It does not define their meaning.
; The host returns the low byte as BitPattern8, never Number.
;
; Required host capability:
;   native-call-bit8-raw
;
; Existing machine layers are reused rather than reimplemented.

(load "lib/core.lisp")
(load "lib/machine/block.lisp")
(load "lib/machine/encoding/x86-64.lisp")
(load "lib/machine/operands/x86-64.lisp")
(load "lib/machine/admission/x86-64.lisp")
(load "lib/machine/atoms/x86-64.lisp")

(def empty-list-cpu-zero-forms
  (lambda ()
    (list
      (x86-xor-r64-r64 (quote rax) (quote rax))
      (x86-ret))))

(def empty-list-cpu-bit-pattern
  (lambda ()
    (native-call-bit8-raw
      (x86-encode-admitted-program
        (empty-list-cpu-zero-forms)))))

; The experimental relation remains Lisp data:
; () is associated with the opaque 8-bit representation 00000000.
(def empty-list-cpu-identity-row
  (list (quote ()) 00000000))

(def empty-list-cpu-bootstrap-witness
  (lambda ()
    (let ((bits (empty-list-cpu-bit-pattern)))
      (cond
        ((eq bits 00000000) (identity-relation same)
         (list
           (quote empty-list-cpu-bootstrap)
           (quote (status pass))
           (list (quote ground) (car empty-list-cpu-identity-row))
           (list (quote representation) bits)
           (list (quote machine-forms) (empty-list-cpu-zero-forms))))
        ((eq bits 00000000) (identity-relation distinct)
         (list
           (quote empty-list-cpu-bootstrap)
           (quote (status fail))
           (list (quote observed) bits)))))))

(empty-list-cpu-bootstrap-witness)
