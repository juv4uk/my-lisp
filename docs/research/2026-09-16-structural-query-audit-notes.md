# Structural query audit frame — 2026-09-16

Issue: #218

The migration rule is domain-first:

> A `?` suffix or historical `t/()` return convention does not grant a question mathematical-binary semantics.

The current public predicate inventory contains 17 surfaces. The #218 audit separates them into producers/consumers and routes each by question domain before any return-shape redesign:

- exact mathematical order/equality: `<`, `=`, `>`, `<=`, `>=` -> #216;
- structural identity/shape/membership: `atom`, `eq`, `equal?`, `member?` -> #218;
- runtime representation/type observations: `numeric-buffer?`, `string?`, `symbol?` -> #218;
- text relations: `string<?`, `string-empty?`, `string-prefix?`, `string-contains?` -> #218;
- generic historical negation: `not` -> #217/#220.

The first executable gate requires exactly one machine-readable domain-classification row for every public predicate and forbids accidental duplication or omission. It deliberately does not ratify the final structural result algebra yet.
