; #218 — knowledge clause classification is explicit domain data.

((expr . "(is-fact? (quote ((planet earth))))")
 (expected . "(clause-kind fact)")
 (active . t)
 (law . empty-body-is-fact))

((expr . "(is-fact? (quote ((ancestor (var x) (var y)) (parent (var x) (var y)))))")
 (expected . "(clause-kind rule)")
 (active . t)
 (law . nonempty-body-is-rule))
