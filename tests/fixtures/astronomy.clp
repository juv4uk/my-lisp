; A small real CLIPS source file, used to prove lib/clips-import.my's
; Step 4 (read-file/read-all) against an actual .clp file on disk, not
; just a caller-supplied quoted literal.
(deffacts initial-facts
  (planet earth)
  (planet mars)
  (star sun))

(defrule orbits-rule
  (planet ?x)
  (star ?y)
  =>
  (assert (orbits ?x ?y)))
