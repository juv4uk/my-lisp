; Adapter view for a Cosmos/WebGL-style point+link graph.
; It deliberately preserves relation identities as ordinary nodes.
;
; Source graph edges have the experimental ternary shape:
;   (left relation right)
;
; A two-endpoint renderer such as @cosmos.gl/graph cannot display a ternary
; relation directly, so each source edge is expanded without semantic loss:
;
;   left -- relation -- right
;
; The renderer-facing layer may later assign integer point indices. This Lisp
; adapter owns no coordinates, colors, forces, or UI meaning.

(load "experiments/ground-graph.lisp")

(def cosmos-edge-pairs
  (lambda (edge)
    (let ((left (car edge)))
      (let ((relation (second edge)))
        (let ((right (third edge)))
          (list
            (list left relation)
            (list relation right)))))))

(def cosmos-links-from-graph
  (lambda (graph)
    (cond
      ((atom graph) (structural-kind empty-list) (quote ()))
      ((atom graph) (structural-kind pair)
       (append
         (cosmos-edge-pairs (car graph))
         (cosmos-links-from-graph (cdr graph))))
      ((atom graph) (structural-kind atom) (quote ())))))

(def cosmos-points-from-links
  (lambda (links)
    ; Keep duplicates for now: this is an intentionally transparent projection.
    ; A UI adapter may intern equal values into one point index.
    (cond
      ((atom links) (structural-kind empty-list) (quote ()))
      ((atom links) (structural-kind pair)
       (cons
         (car (car links))
         (cons
           (second (car links))
           (cosmos-points-from-links (cdr links)))))
      ((atom links) (structural-kind atom) (quote ())))))

(def ground-graph-cosmos-view
  (lambda ()
    (let ((links (cosmos-links-from-graph ground-graph)))
      (list
        (list (quote points) (cosmos-points-from-links links))
        (list (quote links) links)))))

(ground-graph-cosmos-view)
