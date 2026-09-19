; Four-kernel callable graph experiment.
; Based directly on lib/surface/semantic-registry-experiment.lisp and
; experiments/ground-graph.lisp from experiment/empty-list-unfold.
;
; Every endpoint and relation identity is an opaque BitPattern8.
; Rust/C may index and transport this graph but must not assign its meaning.

; Reuse current experimental Lisp semantic IDs where they already exist.
; eval = 01001101
(def lisp-eval-id 01001101)

; Kernel-local entry points for the other islands begin as local opaque IDs.
; Their bit patterns are local to their kernel namespaces.
(def prolog-query-id 00000000)
(def clips-run-id 00000000)
(def datalog-derive-id 00000000)

; Opaque kernel endpoint identities for the graph itself.
; These do not replace per-kernel local function IDs.
(def kernel-lisp 10101000)
(def kernel-prolog 10101001)
(def kernel-clips 10101010)
(def kernel-datalog 10101011)

; Opaque relation identities. Meaning is experimental evidence, not encoded
; in the bits. These values are intentionally outside the current registry
; population and can be revised while this remains an experiment.
(def relation-kernel-has-callable 11110100)
(def relation-can-call 11110101)
(def relation-can-observe 11110110)
(def relation-has-transport 11110111)

; Endpoint form:
;   (kernel-id local-function-id)
; This keeps the same local function bit pattern valid in different kernels.
(def lisp-eval-endpoint (list kernel-lisp lisp-eval-id))
(def prolog-query-endpoint (list kernel-prolog prolog-query-id))
(def clips-run-endpoint (list kernel-clips clips-run-id))
(def datalog-derive-endpoint (list kernel-datalog datalog-derive-id))

; Generic ternary graph shape inherited from ground-graph.lisp:
;   (left relation right)
(def four-kernel-call-graph
  (list
    (list kernel-lisp relation-kernel-has-callable lisp-eval-endpoint)
    (list kernel-prolog relation-kernel-has-callable prolog-query-endpoint)
    (list kernel-clips relation-kernel-has-callable clips-run-endpoint)
    (list kernel-datalog relation-kernel-has-callable datalog-derive-endpoint)

    ; First explicit experimental cross-kernel relationship.
    ; This is relation evidence only; neither endpoint depends on the edge.
    (list lisp-eval-endpoint relation-can-call prolog-query-endpoint)))

; No global root and no universal result object are introduced here.
; Additional edges must be added as experimental evidence, not assumed.

four-kernel-call-graph
