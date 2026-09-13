(
  (name . "addition-is-commutative")
  (types . ("int" "int"))
  (expr . "(= (+ x y) (+ y x))")
)
(
  (name . "string-append-is-associative")
  (types . ("string" "string" "string"))
  (expr . "(equal? (string-append x (string-append y z)) (string-append (string-append x y) z))")
)
(
  (name . "reverse-reverse-is-identity")
  (types . ("list"))
  (expr . "(equal? (reverse (reverse x)) x)")
)
(
  (name . "persistent-map-insert-idempotency")
  (types . ("string-list" "string" "string"))
  (expr . "(let ((m (build-map x map-empty))) (equal? (map-insert y z (map-insert y z m)) (map-insert y z m)))")
)
(
  (name . "persistent-map-sorted-order")
  (types . ("string-list"))
  (expr . "(let ((m (build-map x map-empty))) (map-keys-sorted? (map->list m)))")
)
(
  (name . "persistent-map-height-bound")
  (types . ("string-list"))
  (expr . "(let ((m (build-map x map-empty))) (let ((h (height-of m)) (n (length (map->list m)))) (>= n (+ (fib (+ h 2)) -1))))")
)
(
  (name . "world-monotonicity")
  (types . ("string-list" "string" "string"))
  (expr . "(let ((w (build-world x (empty-world)))) (= (+ 1 (length (world-journal w))) (length (world-journal (world-tell w y z)))))")
)
(
  (name . "world-rollback-identity")
  (types . ("string-list" "string" "string"))
  (expr . "(let ((w (build-world x (empty-world)))) (equal? (world-parent (world-tell w y z)) w))")
)
(
  (name . "knowledge-journal-append-only-monotonicity")
  (types . ("string-list" "string" "string"))
  (expr . "(let ((_1 (def *knowledge-journal* (quote ()))) (_2 (tell-all-events x)) (len-before (length *knowledge-journal*)) (_3 (defmodule \"mod\" (list z)))) (= (+ 1 len-before) (length *knowledge-journal*)))")
)
(
  (name . "knowledge-journal-retract-inverse-of-tell")
  (types . ("string-list" "string"))
  (expr . "(let ((_1 (def *knowledge-journal* (quote ()))) (_2 (tell-all-events x)) (before (module-clauses-now \"mod\")) (_3 (defmodule \"mod\" (list y))) (_4 (retract-knowledge \"mod\" y))) (equal? before (module-clauses-now \"mod\")))")
)
