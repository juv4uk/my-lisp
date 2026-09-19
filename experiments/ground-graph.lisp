; Experimental ground graph.
; No registry rows, columns, surface tags, or Canon live here.
; Every machine identity/relation marker is an opaque BitPattern8.
;
; Apparatus still introduced by the current Lisp runtime:
; list/pair traversal, atom/eq/cond, and the reader.
; The experiment asks whether one generic graph mechanism can express both
; equivalence and transition without knowing the meaning of endpoint values.

; E-class-inspired peer set: two representations coexist without destructive
; conversion. This is intentionally tiny; it borrows the e-class idea, not a
; full equality-saturation engine.
(def ground-eclass
  (list (quote ()) 00000000))

(def two-member-eclass-peer
  (lambda (class value)
    (let ((left (car class)))
      (let ((right (second class)))
        (cond
          ((eq value left) (identity-relation same) right)
          ((eq value left) (identity-relation distinct)
           (cond
             ((eq value right) (identity-relation same) left)
             ((eq value right) (identity-relation distinct) (quote ())))))))))

; Opaque relation identities. Their meanings are not encoded in their bits.
(def ground-equivalence-relation 00000010)
(def ground-transition-relation 00000011)

; Generic edge shape: (endpoint relation endpoint).
; An edge is traversable in either direction by the same mechanism.
(def ground-graph
  (list
    (list (quote ()) ground-equivalence-relation 00000000)
    (list 00000000 ground-transition-relation 00000001)))

(def graph-edge-neighbor
  (lambda (edge relation value)
    (let ((left (car edge)))
      (let ((edge-relation (second edge)))
        (let ((right (third edge)))
          (cond
            ((eq edge-relation relation) (identity-relation same)
             (cond
               ((eq left value) (identity-relation same) right)
               ((eq left value) (identity-relation distinct)
                (cond
                  ((eq right value) (identity-relation same) left)
                  ((eq right value) (identity-relation distinct) (quote ()))))))
            ((eq edge-relation relation) (identity-relation distinct)
             (quote ()))))))))

(def graph-neighbor
  (lambda (graph relation value)
    (cond
      ((atom graph) (structural-kind empty-list) (quote ()))
      ((atom graph) (structural-kind pair)
       (let ((candidate (graph-edge-neighbor (car graph) relation value)))
         (cond
           ((atom candidate) (structural-kind empty-list)
            (graph-neighbor (cdr graph) relation value))
           ((atom candidate) (structural-kind atom) candidate)
           ((atom candidate) (structural-kind pair) candidate))))
      ((atom graph) (structural-kind atom) (quote ())))))

(def ground-graph-witness
  (lambda ()
    (list
      (list
        (quote eclass-forward)
        (two-member-eclass-peer ground-eclass (quote ())))
      (list
        (quote eclass-backward)
        (two-member-eclass-peer ground-eclass 00000000))
      (list
        (quote graph-equivalence-forward)
        (graph-neighbor ground-graph ground-equivalence-relation (quote ())))
      (list
        (quote graph-equivalence-backward)
        (graph-neighbor ground-graph ground-equivalence-relation 00000000))
      (list
        (quote graph-transition-forward)
        (graph-neighbor ground-graph ground-transition-relation 00000000))
      (list
        (quote graph-transition-backward)
        (graph-neighbor ground-graph ground-transition-relation 00000001)))))

(ground-graph-witness)


; ---------------------------------------------------------------------------
; Nodum-inspired experiments: unresolved endpoints, late resolution, inverse
; queries, and bounded observation.
;
; Important repair over graph-neighbor above:
; () is a legitimate graph endpoint, so it must never double as "not found".
; These queries therefore return an explicit structural envelope:
;   (graph-result found VALUE)
;   (graph-result absent)
; The words are apparatus labels only; they are not kernel identities.

(def graph-result-found
  (lambda (value)
    (list (quote graph-result) (quote found) value)))

(def graph-result-absent
  (lambda ()
    (list (quote graph-result) (quote absent))))

(def graph-result-found?
  (lambda (result)
    (eq (second result) (quote found))))

(def graph-result-value
  (lambda (result)
    (third result)))

; Same edge shape as above, but with an unambiguous result envelope.
(def graph-edge-neighbor-result
  (lambda (edge relation value)
    (let ((left (car edge)))
      (let ((edge-relation (second edge)))
        (let ((right (third edge)))
          (cond
            ((eq edge-relation relation) (identity-relation same)
             (cond
               ((eq left value) (identity-relation same)
                (graph-result-found right))
               ((eq left value) (identity-relation distinct)
                (cond
                  ((eq right value) (identity-relation same)
                   (graph-result-found left))
                  ((eq right value) (identity-relation distinct)
                   (graph-result-absent))))))
            ((eq edge-relation relation) (identity-relation distinct)
             (graph-result-absent))))))))

(def graph-neighbor-result
  (lambda (graph relation value)
    (cond
      ((atom graph) (structural-kind empty-list)
       (graph-result-absent))
      ((atom graph) (structural-kind pair)
       (let ((candidate (graph-edge-neighbor-result (car graph) relation value)))
         (cond
           ((graph-result-found? candidate) (identity-relation same)
            candidate)
           ((graph-result-found? candidate) (identity-relation distinct)
            (graph-neighbor-result (cdr graph) relation value)))))
      ((atom graph) (structural-kind atom)
       (graph-result-absent)))))

; Opaque handle whose concrete peer is intentionally absent at first.
; "Unresolved" is represented by the absence of a resolution edge, not by a
; special host type or magic flag.
(def late-endpoint 00000100)
(def late-representation 00000101)
(def late-relation 00000110)
(def resolution-relation 00000111)

; The asserted graph is immutable across both observations.
(def late-binding-graph
  (list
    (list 00000001 late-relation late-endpoint)))

; Before: there is no resolution evidence.
(def resolution-before (quote ()))

; After: new evidence relates the old handle to a concrete peer. The asserted
; edge above is unchanged.
(def resolution-after
  (list
    (list late-endpoint resolution-relation late-representation)))

(def resolve-endpoint
  (lambda (resolution-graph endpoint)
    (let ((resolution
            (graph-neighbor-result
              resolution-graph
              resolution-relation
              endpoint)))
      (cond
        ((graph-result-found? resolution) (identity-relation same)
         resolution)
        ((graph-result-found? resolution) (identity-relation distinct)
         (graph-result-found endpoint))))))

; Traverse the asserted edge first, then resolve its endpoint using a separate
; graph. This is late binding: resolution data changes, source evidence does not.
(def graph-neighbor-resolved
  (lambda (graph resolution-graph relation value)
    (let ((neighbor (graph-neighbor-result graph relation value)))
      (cond
        ((graph-result-found? neighbor) (identity-relation same)
         (resolve-endpoint resolution-graph (graph-result-value neighbor)))
        ((graph-result-found? neighbor) (identity-relation distinct)
         neighbor)))))

; Directed inverse query. This derives a backlink-like view from the one stored
; source->target edge; it does not add a second authoritative edge.
(def graph-first-source-result
  (lambda (graph relation target)
    (cond
      ((atom graph) (structural-kind empty-list)
       (graph-result-absent))
      ((atom graph) (structural-kind pair)
       (let ((edge (car graph)))
         (let ((source (car edge)))
           (let ((edge-relation (second edge)))
             (let ((edge-target (third edge)))
               (cond
                 ((eq edge-relation relation) (identity-relation same)
                  (cond
                    ((eq edge-target target) (identity-relation same)
                     (graph-result-found source))
                    ((eq edge-target target) (identity-relation distinct)
                     (graph-first-source-result (cdr graph) relation target))))
                 ((eq edge-relation relation) (identity-relation distinct)
                  (graph-first-source-result (cdr graph) relation target))))))))
      ((atom graph) (structural-kind atom)
       (graph-result-absent)))))

; Bounded observation uses structural budget instead of arithmetic depth.
; One list cell authorizes one graph hop. The budget is observer apparatus,
; not an intrinsic property of the graph.
(def graph-observe-path
  (lambda (graph relation value budget)
    (cond
      ((atom budget) (structural-kind empty-list)
       (list value))
      ((atom budget) (structural-kind pair)
       (let ((next (graph-neighbor-result graph relation value)))
         (cond
           ((graph-result-found? next) (identity-relation same)
            (cons value
                  (graph-observe-path
                    graph
                    relation
                    (graph-result-value next)
                    (cdr budget))))
           ((graph-result-found? next) (identity-relation distinct)
            (list value))))
      ((atom budget) (structural-kind atom)
       (list value))))))

; A separate tiny chain makes bounded observation deterministic: each node has
; at most one neighbor under this relation in the forward-looking test data.
(def observation-relation 00001000)
(def observation-graph
  (list
    (list 00001010 observation-relation 00001011)
    (list 00001001 observation-relation 00001010)))

(def nodum-kernel-lessons-witness
  (lambda ()
    (list
      (list
        (quote resolution-evidence-before)
        (graph-neighbor-result
          resolution-before
          resolution-relation
          late-endpoint))
      (list
        (quote unresolved-before)
        (graph-neighbor-resolved
          late-binding-graph
          resolution-before
          late-relation
          00000001))
      (list
        (quote resolved-after)
        (graph-neighbor-resolved
          late-binding-graph
          resolution-after
          late-relation
          00000001))
      (list
        (quote asserted-edge-still)
        (graph-neighbor-result
          late-binding-graph
          late-relation
          00000001))
      (list
        (quote derived-backlink)
        (graph-first-source-result
          late-binding-graph
          late-relation
          late-endpoint))
      (list
        (quote bounded-one-hop)
        (graph-observe-path
          observation-graph
          observation-relation
          00001001
          (quote (step))))
      (list
        (quote bounded-two-hop)
        (graph-observe-path
          observation-graph
          observation-relation
          00001001
          (quote (step step)))))))



; Generic e-class membership: no fixed language/surface columns and no
; two-member limit. A class is ordinary Lisp data containing peer
; representations.
(def eclass-member-result
  (lambda (class value)
    (cond
      ((atom class) (structural-kind empty-list)
       (graph-result-absent))
      ((atom class) (structural-kind pair)
       (cond
         ((eq (car class) value) (identity-relation same)
          (graph-result-found class))
         ((eq (car class) value) (identity-relation distinct)
          (eclass-member-result (cdr class) value))))
      ((atom class) (structural-kind atom)
       (graph-result-absent)))))

(def expanded-ground-eclass
  (list
    (quote ())
    00000000
    00001100
    00001101))

; Persistent graph growth: each generation adds relation evidence without
; destroying the previous graph value. The earlier generation is recoverable
; structurally as the tail of the new one.
(def graph-generation-0 (quote ()))

(def graph-generation-1
  (cons
    (list (quote ()) ground-equivalence-relation 00000000)
    graph-generation-0))

(def graph-generation-2
  (cons
    (list 00000000 ground-transition-relation 00000001)
    graph-generation-1))

(def graph-growth-witness
  (lambda ()
    (list
      (list
        (quote expanded-class-member)
        (eclass-member-result expanded-ground-eclass 00001101))
      (list
        (quote previous-generation)
        (cdr graph-generation-2))
      (list
        (quote previous-generation-preserved)
        (eq (cdr graph-generation-2) graph-generation-1)))))



; ---------------------------------------------------------------------------
; Basalt-inspired relation lifecycle.
;
; Candidate, evidence, assertion, and resolution are deliberately kept as
; separate graph layers. None is a privileged host type. The BitPattern8 values
; below are opaque relation identities used only by this experiment.

(def lifecycle-candidate-relation 00001110)
(def lifecycle-evidence-relation 00001111)
(def lifecycle-asserted-relation 00010000)
(def lifecycle-resolution-relation 00010001)

(def lifecycle-source 00010010)
(def lifecycle-handle 00010011)
(def lifecycle-representation 00010100)

; A candidate exists first. This alone is not enough to produce an assertion.
(def lifecycle-candidate-graph
  (list
    (list
      lifecycle-source
      lifecycle-candidate-relation
      lifecycle-handle)))

; No evidence yet.
(def lifecycle-evidence-before (quote ()))

; Evidence may appear later without rewriting the candidate graph.
(def lifecycle-evidence-after
  (list
    (list
      lifecycle-source
      lifecycle-evidence-relation
      lifecycle-handle)))

; Resolution is independent again: an asserted endpoint may still be unresolved.
(def lifecycle-resolution-before (quote ()))

(def lifecycle-resolution-after
  (list
    (list
      lifecycle-handle
      lifecycle-resolution-relation
      lifecycle-representation)))

; Derive one asserted edge only when candidate and evidence independently name
; the same source/target pair. The candidate graph itself is never mutated.
(def derive-asserted-edge
  (lambda (candidate-graph evidence-graph source)
    (let ((candidate
            (graph-neighbor-result
              candidate-graph
              lifecycle-candidate-relation
              source)))
      (cond
        ((graph-result-found? candidate) (identity-relation same)
         (let ((evidence
                 (graph-neighbor-result
                   evidence-graph
                   lifecycle-evidence-relation
                   source)))
           (cond
             ((graph-result-found? evidence) (identity-relation same)
              (cond
                ((eq
                   (graph-result-value candidate)
                   (graph-result-value evidence))
                 (identity-relation same)
                 (graph-result-found
                   (list
                     source
                     lifecycle-asserted-relation
                     (graph-result-value candidate))))
                ((eq
                   (graph-result-value candidate)
                   (graph-result-value evidence))
                 (identity-relation distinct)
                 (graph-result-absent))))
             ((graph-result-found? evidence) (identity-relation distinct)
              (graph-result-absent)))))
        ((graph-result-found? candidate) (identity-relation distinct)
         (graph-result-absent))))))

; Resolve the target of a derived asserted edge, if independent resolution
; evidence exists. Absence of resolution does not revoke the assertion.
(def resolve-asserted-edge
  (lambda (asserted-result resolution-graph)
    (cond
      ((graph-result-found? asserted-result) (identity-relation same)
       (let ((edge (graph-result-value asserted-result)))
         (let ((target (third edge)))
           (let ((resolved
                   (graph-neighbor-result
                     resolution-graph
                     lifecycle-resolution-relation
                     target)))
             (cond
               ((graph-result-found? resolved) (identity-relation same)
                (graph-result-found
                  (list
                    (car edge)
                    (second edge)
                    (graph-result-value resolved))))
               ((graph-result-found? resolved) (identity-relation distinct)
                asserted-result))))))
      ((graph-result-found? asserted-result) (identity-relation distinct)
       asserted-result))))

(def relation-lifecycle-witness
  (lambda ()
    (let ((before
            (derive-asserted-edge
              lifecycle-candidate-graph
              lifecycle-evidence-before
              lifecycle-source)))
      (let ((after
              (derive-asserted-edge
                lifecycle-candidate-graph
                lifecycle-evidence-after
                lifecycle-source)))
        (list
          (list
            (quote candidate-visible)
            (graph-neighbor-result
              lifecycle-candidate-graph
              lifecycle-candidate-relation
              lifecycle-source))
          (list
            (quote assertion-before-evidence)
            before)
          (list
            (quote assertion-after-evidence)
            after)
          (list
            (quote unresolved-assertion)
            (resolve-asserted-edge
              after
              lifecycle-resolution-before))
          (list
            (quote resolved-assertion)
            (resolve-asserted-edge
              after
              lifecycle-resolution-after))
          (list
            (quote candidate-preserved)
            (graph-neighbor-result
              lifecycle-candidate-graph
              lifecycle-candidate-relation
              lifecycle-source)))))))

