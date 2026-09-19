; Four-kernel callable graph experiment.
; Addresses come from lib/surface/semantic-registry-archipelago-experiment.lisp.
; Address shape: (namespace8 local-id8).
;
; Shared graph relation addresses live in namespace 00000000.
; Kernel-local callable addresses live in their owning namespace.

(def address-lisp-eval (list 00000001 01001101))
(def address-prolog-query (list 00000010 00000001))
(def address-clips-run (list 00000011 00000111))
(def address-datalog-derive (list 00000100 00001010))

(def relation-owns-callable (list 00000000 00011010))
(def relation-can-call (list 00000000 00011011))
(def relation-can-observe (list 00000000 00011100))
(def relation-has-transport (list 00000000 00011101))

; Generic ternary graph inherited from ground-graph.lisp:
;   (left-address relation-address right-address)
(def four-kernel-call-graph
  (list
    ; First explicit cross-kernel experiment:
    ; Lisp eval may call Prolog query through an explicit relation.
    (list address-lisp-eval relation-can-call address-prolog-query)

    ; All four callables can later acquire transport edges independently.
    (list address-lisp-eval relation-has-transport address-lisp-eval)
    (list address-prolog-query relation-has-transport address-prolog-query)
    (list address-clips-run relation-has-transport address-clips-run)
    (list address-datalog-derive relation-has-transport address-datalog-derive)))

four-kernel-call-graph
