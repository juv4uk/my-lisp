# Structural query inventory witness

`structural-query-inventory-witness.lisp` owns the completeness verdict for #218.
The Rust test transports Lisp data documents only; it does not classify predicates.

The witness derives the public predicate set from `lib/surface/uk-inventory.lisp` and requires one row per predicate in `contracts/structural-query-inventory.lisp`.
