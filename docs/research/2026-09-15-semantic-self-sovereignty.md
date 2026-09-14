# Semantic self-sovereignty — design capital

**Date:** 2026-09-15  
**Status:** design capital; status vocabulary below is normative for this note  
**Verified against:** `main` at `8bd551cc5f4282db881f2963a756a6e394f30a7e`  
**Scope:** authority boundaries, evidence discipline, and future-proofing. This note does not change evaluator semantics or authorize new runtime features.

## One-sentence center

> **Semantic self-sovereignty** means that the language owns the normative definition of its observable meaning, while evaluators, compilers, hosts, accelerators, and hardware are replaceable implementations that must earn admission by evidence.

This is deliberately stronger and more precise than `self-hosting`. Self-hosting asks *what implementation language executes the evaluator?* Semantic self-sovereignty asks *who is allowed to decide what the program means?*

## Status vocabulary for every arrow

Every architectural arrow named in this note has exactly one status:

- **RATIFIED** — executable evidence exists and the claim is limited to what that evidence proves.
- **HYPOTHESIS** — a falsifiable milestone can be stated, but the evidence does not yet exist.
- **ILLUSTRATION, NOT COMMITMENT** — a possible direction used to explain the architecture. It does not create roadmap scope merely by appearing in a diagram.

There is no fourth state in which an attractive diagram silently becomes a commitment.

## Ratified core

### Peer semantic surfaces — RATIFIED

Surface spellings are projections of one semantic identity; they are not translations from a privileged English semantic core. Examples such as `car`, `перше`, `ādi`, and symbolic peers converge on the same semantic identity through the registry.

```text
surface spelling
      ↓
semantic identity
      ↓
observable meaning
```

A surface may vary. The semantic identity must not drift because a surface varies.

### Compiler authority boundary — RATIFIED as a boundary

`docs/COMPILER-AUTHORITY-BOUNDARY.md` already states the core rule:

> Lisp defines meaning; a compiler may change representation and execution strategy, but must not invent or silently replace language semantics.

A compiler or optimizer is therefore an **untrusted optimizing candidate** until its outputs are checked against language-owned semantic evidence.

For CML, the defensible present-tense statement is:

> **CML is an untrusted optimizing candidate checked against Lisp-owned semantic evidence.**

Do not call the current state "certified compilation" or full "translation validation". Those terms become justified only when a validator checks a preservation relation for the concrete source → artifact transition, rather than merely checking end outcomes on a corpus.

### Vertical machine path — RATIFIED, bounded

The ratified Vertical Day record proves a bounded path from Lisp-owned structural meaning through admitted structured machine forms, Lisp-owned x86 encoding, a semantics-blind W^X host mechanism, and physical CPU execution. The proof is deliberately narrower than a full native Lisp runtime.

It does **not** establish a native GC, escaping first-class native pairs, a complete native heap, automatic heterogeneous scheduling, or a complete Lisp machine.

## Current trust root

A corpus can be the sole normative source of expected semantic facts while still depending on an implementation to parse and execute the corpus. These are different claims and must not be conflated.

### What is trusted today

Today, `tests/fixtures/conformance.lisp` is being promoted toward the single language-owned source of expected semantic facts, and `lib/witness.lisp` is being developed in #113 as a Lisp-owned comparator. However, the current system still depends on implementation substrate — including the native Rust evaluator and reader/runtime machinery — to execute that Lisp and observe outcomes.

Therefore:

> **The native Rust evaluator is not declared the semantic oracle, but it remains part of the current trusted execution root.**

The data authority and the execution trust root are separate axes.

### Trust-minimization direction

The intended reduction is:

```text
unchanged Lisp-owned corpus
          ↓
independent consumers / evaluators
          ↓
disagreement treated as evidence, never normalized away
          ↓
smaller explicitly named trusted semantic kernel
```

An independent implementation does not become correct merely by disagreeing with Rust, and Rust does not become correct merely because it is first. A disagreement is a sensor: it forces the project to identify which assumption or representation differs and which evidence actually settles the semantic fact.

### Claim discipline

Until independent consumers can execute the same unchanged witnesses with sufficiently independent implementation assumptions, do not claim that the trust root has disappeared. The goal is to **minimize and name** the trusted root, not to hide it behind the word "Lisp".

## Performance non-interference

Performance policy is explicitly downstream of semantics.

```text
implementation A passes semantic witnesses
implementation B passes semantic witnesses
                 ↓
semantically equal admitted candidates
                 ↓
performance / energy / size policy selects A or B
                 ↓
NO backward edge into semantic meaning
```

Rules:

1. Passing the same semantic witnesses makes admitted implementations semantically equivalent **for the covered contract**, not equally fast.
2. Benchmarks, CPU profiles, crossover thresholds, scheduler heuristics, and target capabilities are policy/mechanism evidence.
3. A faster implementation does not become more canonical, more semantically true, or a new semantic identity because it is faster.
4. Changing a performance selector must not edit semantic IDs, semantic expected values, or the language-owned witness definitions merely to preserve the selector's preferred outcome.
5. If an optimization cannot pass the existing witnesses, the optimization is rejected or the language contract is changed explicitly through the semantic authority process. Performance never changes the contract by backpressure.

This is the preventive form of the same lesson behind keeping machine facts and target-specific mechanisms out of the semantic registry.

## Error Observation Identity v1

This note names, but does not expand, the boundary already ratified by ADR-011.

**Error Observation Identity v1:**

```text
cross-runtime semantic error identity
               =
admitted error category
```

Today, `UnknownSymbol`, `Arity`, `Type`, `InvalidForm`, `OutOfMemory`, `NumericOverflow`, `DivisionByZero`, and `Parse` are the admitted cross-runtime categories exposed by the current contract.

The following are **not** part of cross-runtime semantic error identity v1 unless a future contract explicitly ratifies them:

- human-readable message text;
- source span / byte offsets;
- meta-evaluator `detail` payload;
- implementation-internal exception/control-flow shape;
- payload spellings such as the particular unbound symbol name.

Therefore `(error unbound-symbol x)` and `(error unbound-symbol y)` belong to the same current cross-runtime semantic error class when both normalize to the same admitted category. The `x`/`y` payload remains useful diagnostic data, but it is not currently a ratified identity-relevant field.

A future detail field may become semantic only through all three gates:

1. explicit ADR/contract decision;
2. implementation-neutral machine-readable schema;
3. executable witnesses shared by implementations.

No consumer may promote an implementation detail into semantic identity merely because that detail is convenient to assert.

## Witness authority program — HYPOTHESIS in progress

The #113 → #116 sequence is the immediate authority program:

```text
#113  Lisp-owned executable witness authority
  ↓
#114  classify/relocate semantic truth out of host assertions
  ↓
#115  enforce the authority boundary
  ↓
#116  stable backend-neutral actual-outcome transport
```

The intended result is that native, meta, CML, and later implementations submit actual outcomes to the same language-owned witness definitions. A backend may support a smaller admitted slice, but it does not receive a private expected-results table.

This sequence is **not yet fully ratified** merely because its design is accepted. Each step earns its status through its own executable evidence.

## #145 HOST-SURGERY-1 — future falsification gate

**Status: HYPOTHESIS; blocked on #116.**

Purpose: test the statement "semantic truth survives an implementation-internal change" without changing the semantic witness authority.

Required shape:

```text
record corpus/witness digest D
          ↓
perform implementation-internal host surgery
          ↓
CI asserts corpus/witness digest == D
          ↓
run unchanged corpus through the affected implementation
          ↓
all applicable semantic witnesses remain GREEN
```

The digest pin is mandatory. "We did not edit expected truth" must be machine-checkable, not a promise in prose.

The surgery must be mechanical rather than semantic — examples that may be considered later include pair representation, environment layout, internal traversal strategy, or another implementation detail that the existing authority boundary already permits to vary. The exact surgery is intentionally not chosen in this design note; choosing it belongs to #145 after #116 establishes transport.

A surgery that requires editing semantic expected values to recover GREEN is a falsification of the claimed separation, not a successful migration.

## Experimental stands

These are hypotheses or illustrations, not automatically scheduled work.

| Stand | Question | Status |
|---|---|---|
| CML | Can an optimizing compiler change execution strategy while preserving Lisp-owned meaning? | HYPOTHESIS moving toward stronger evidence |
| `wsm-my-lisp` | Can more of the machine path be owned by Lisp while keeping the host semantics-blind? | HYPOTHESIS with ratified bounded witnesses |
| FPGA | Can the same semantic authority survive a custom hardware implementation? | HYPOTHESIS |
| GPU / SIMD | Can heterogeneous acceleration be selected purely as policy after semantic admission? | ILLUSTRATION, NOT COMMITMENT until a milestone is separately accepted |
| `my-lisp-cyberpunk` | Can Lisp command a large foreign runtime without delegating semantic authority to it? | HYPOTHESIS / separate project evidence |
| `wsm-os-lisp` | Can Lisp become a system environment while preserving the same authority boundary? | ILLUSTRATION, NOT COMMITMENT until separately ratified |

No feature is justified because it "fits the picture". Each transition requires its own falsifiable milestone and witness.

## Prior-art discipline

Prior-art conclusions remain owned by #132 and Rule 15, not by this note. The research queue should include, without presuming similarity or novelty:

- golden/reference-model validation in hardware design;
- Sail / executable ISA models, including RISC-V-related use;
- CompCert and adjacent verified-compilation work;
- translation-validation literature;
- WebAssembly conformance across independent engines;
- Java TCK/JCK-style conformance;
- **definitional interpreters and semantics-first systems, including CakeML, the K Framework, and PLT Redex**.

Each is only a candidate row until checked with date, exact queries, primary/strong sources, overlap dimensions, missing dimensions, and a Rule-15 verdict.

## Claims made

The project may defensibly say:

- language semantics and mechanical execution are intentionally separated by an explicit authority boundary;
- peer language surfaces can map to one semantic identity rather than forming a translation hierarchy;
- bounded structural Lisp semantics have reached physical x86-64 execution through a Lisp-owned admitted lowering/encoding path and a semantics-blind host;
- the project is actively relocating expected semantic truth from host-authored assertions into Lisp-owned executable witnesses;
- performance selection is subordinate to semantic admission, by design.

## Claims not made

This note does **not** claim:

- that Rust has ceased to be part of the trusted execution root;
- that #113–#116 are complete merely because their architecture is accepted;
- that CML is a certified compiler or that current corpus checks constitute full translation validation;
- that my-lisp has a complete native runtime, GC, first-class escaping native object model, GPU scheduler, FPGA Lisp machine, or Lisp OS;
- that error payload/detail fields are cross-runtime semantic identity today;
- that any listed prior-art system is weaker, stronger, equivalent, or a counterexample before #132 records evidence;
- that my-lisp is unique among all existing systems.

## Research principle

> **An implementation earns a semantic arrow by passing unchanged language-owned evidence. A diagram, benchmark, implementation language, or machine target cannot grant itself semantic authority.**

The center is not a particular evaluator, compiler, or processor. It is the attempt to keep a stable, executable notion of meaning while the mechanisms underneath it are allowed to change — and to make every claimed separation falsifiable.