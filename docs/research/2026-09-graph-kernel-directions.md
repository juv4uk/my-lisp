# Research note: graph rewriting, reversible computing, and e-graphs

Status: experimental research input, not language authority.

This note records ideas investigated in September 2026 for the small parallel
kernel experiment around `()`, opaque `BitPattern8` identities, and direct
machine execution.

## 1. Graph rewriting

Interaction nets are a restricted graph-rewriting model. Their appeal for this
experiment is locality: computation can be described as matching a small piece
of graph and replacing it, rather than evaluating a large syntax tree.

Relevant recent work:

- Shinya Sato, *Conditional Nested Pattern Matching in Interaction Net* (2024):
  https://arxiv.org/abs/2410.00540

Experimental lesson:

```text
graph state
  -> local relation/rule
  -> graph state
```

Do not infer Lisp semantics from CPU instructions. Machine instructions may
realize a graph transition without naming its meaning.

## 2. Reversible computing

A useful stronger criterion for an experimental transition is not merely that it
can run forward, but that the resulting state retains enough information for a
corresponding inverse transition.

Relevant recent work:

- Ivan Lanese, Germán Vidal, *A Reversible Semantics for Janus* (2026):
  https://arxiv.org/abs/2602.16913
- Pablo Arrighi, Marin Costes, Luidnel Maignan,
  *Space-time reversible graph rewriting* (2025):
  https://arxiv.org/abs/2510.03296

Experimental lesson:

```text
A --R--> B
A <--R'-- B
```

For the first hardware witness we can test this physically:

```text
00000000 --BTS bit0--> 00000001
00000001 --BTR bit0--> 00000000
```

These are bit-pattern transitions, not arithmetic claims about 0 and 1.

## 3. E-graphs / equality saturation

E-graphs maintain equivalence classes without forcing one representation to
replace another. That suggests a better model for the ground relation than a
one-way conversion.

Relevant recent work:

- Jules Merckx et al., *E-Graphs as a Persistent Compiler Abstraction* (2026):
  https://arxiv.org/abs/2602.16707
- Sijie Kong et al., *Improving Equality Saturation for EDA via Semantic
  E-Graphs* (PLDI 2026):
  https://pldi26.sigplan.org/details/pldi-2026-papers/56/Improving-Equality-Saturation-for-EDA-via-Semantic-E-Graphs
- *Versioned E-Graphs* (PLDI 2026):
  https://pldi26.sigplan.org/details/pldi-2026-papers/6/Versioned-E-Graphs
- EGRAPHS 2026 workshop:
  https://pldi26.sigplan.org/home/egraphs-2026

The semantic-e-graph work is especially relevant because it explicitly bridges
high-level values and low-level bit representations.

Experimental lesson:

```text
E0 = { (), 00000000 }
```

This is stronger and cleaner than treating `() -> 00000000` as a destructive
conversion. The structural representation and the physical 8-bit
representation can coexist as peers in one equivalence class.

## 4. Constraint: do not import full e-graph machinery yet

Modern equality saturation can grow memory aggressively. EGRAPHS 2026 includes
work specifically on memory-scalable scheduling:

- *CERES: Making Equality Saturation Memory-Scalable*:
  https://pldi26.sigplan.org/details/egraphs-2026-papers/2/CERS-Making-Equality-Saturation-Memory-Scalable

Therefore this experiment should borrow the *idea* of equivalence classes, not
install a full equality-saturation engine.

## 5. Current experimental model

Use existing my-lisp/Rust/x86 code as apparatus, but keep the experimental
meaning outside the old Canon.

```text
E0 = { (), 00000000 }

00000000 --set-bit-0--> 00000001
00000001 --clear-bit-0--> 00000000
```

Physical realization:

```text
XOR RAX,RAX ; materialize zeroed bit pattern
BTS RAX,0   ; forward transition
BTR RAX,0   ; inverse transition
RET
```

The CPU knows only bits and transitions. It does not know "empty list",
"identity", "successor", or arithmetic zero/one.

## 6. Provenance discipline

For every experiment distinguish:

- GIVEN: `()`
- INTRODUCED DATA: opaque BitPattern8 values and graph relations
- APPARATUS: reader, evaluator, Rust value representation, machine atoms,
  admission, encoder, native executor, CPU
- DERIVED OBSERVATION: the exact runtime relation/transition demonstrated by
  the witness

The goal is to shrink INTRODUCED assumptions over time, not to hide them.

## 7. Immediate experiments

1. Ground e-class: prove both directions `() <-> 00000000` through one generic
   class-membership mechanism.
2. Reversible physical edge: execute `00000000 -> 00000001 -> 00000000` using
   admitted BTS/BTR forms and preserve results as BitPattern8.
3. Graph rewrite prototype: represent the physical transition as graph data and
   traverse it without special knowledge of the identities' meaning.
4. Only after those are stable, test whether rules themselves can become graph
   data instead of host/evaluator code.

No claim of minimality or self-bootstrap is made yet.
