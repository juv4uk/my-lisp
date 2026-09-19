; Four-kernel callable graph experiment.
; Based on the single 8-bit semantic-registry-experiment.lisp address space.
;
; The experiment asks whether <=256 primitive identities are enough to operate
; Lisp, Prolog, CLIPS and Datalog together.
;
; Therefore kernel interaction does NOT get a wider address type.
; A primitive may be shared across kernels or interpreted locally through an
; explicit graph relation/endpoint, but every primitive identity remains BitPattern8.

; Existing Lisp eval primitive.
(def lisp-eval-id 01001101)

; Four kernel endpoint identities from semantic-registry-experiment.lisp.
(def kernel-lisp 10111101)
(def kernel-prolog 10111110)
(def kernel-clips 10111111)
(def kernel-datalog 11000000)

; Shared interaction relation identities from the same 8-bit table.
(def relation-kernel-has-callable 11000001)
(def relation-can-call 11000010)
(def relation-can-observe 11000011)
(def relation-has-transport 11000100)

; A callable endpoint is ordinary Lisp data. The operation identity itself
; remains a single 8-bit primitive identity.
(def lisp-eval-endpoint (list kernel-lisp lisp-eval-id))

; Prolog / CLIPS / Datalog are intentionally NOT assigned complete private
; vocabularies here. The experiment is to discover the smallest shared set of
; 8-bit primitives sufficient to operate all four kernels.

; Generic ternary graph inherited from ground-graph.lisp:
;   (left relation right)
(def four-kernel-call-graph
  (list
    (list kernel-lisp relation-kernel-has-callable lisp-eval-endpoint)))

four-kernel-call-graph
