# GC current-runtime audit — #153

Status: evidence/audit only; no collector implementation.
Date: 2026-09-15

## Why this audit exists

The 2026-08-23 GC consensus selected a Pair-only first managed slice. The current runtime has since changed materially: semantic callable identity is moving toward `SemanticRef`, recursive definitions use shared lexical environments, and the machine/backend work now needs a portable distinction between semantic identity, runtime object identity, and physical address.

This document records the current ownership facts before #154 changes storage.

## Current ownership graph

```text
Value
├─ immediate / non-graph semantic values
│  └─ SemanticRef(id)          semantic identity; never heap identity
├─ Pair                        Rc<Value> × Rc<Value>
├─ Closure / Macro             Rc<Closure> → captured Environment
├─ Vector                      Rc<RefCell<Vec<Value>>>
├─ Symbol / String             Rc<str>
├─ Rational / BigInt           immutable numeric payload
└─ host/resource payloads      explicit host lifecycle; not GC-finalizer authority

Environment
└─ Rc<RefCell<Frame>>
   ├─ values: name → Value
   └─ parent: Option<Environment>
```

The current `Environment` already has an iterative `Drop` and a 300,000-frame regression test. Therefore deep parent-chain destruction is **not** a current reason to introduce tracing GC.

## Current reasons for tracing GC

1. **Cycles.** A closure captures an `Environment`; that environment can bind the closure. Shared recursive definition frames make this graph natural rather than exceptional.
2. **Mutable graph objects.** `Vector` can retain Values and therefore participate in cycles once mutation exposes a back-edge.
3. **Portable runtime identity.** Future CML/FPGA object memory must not inherit Rust `Rc` pointer identity as a language fact.

Canonical identity separation:

```text
SemanticRef(id) != ObjectId(slot,generation) != physical address
```

- `SemanticRef` answers **what operation/meaning is this?**
- `ObjectId` will answer **which managed runtime object is this?**
- physical address is backend mechanism only.

## Classification for #153

| Runtime class | Current storage | M0 classification | Reason |
|---|---|---|---|
| Nil/Bool/simple numeric immediates | inline | non-traced | no graph edges |
| SemanticRef | semantic ID | non-traced semantic identity | must not acquire heap/pointer authority |
| Pair | Rc graph | managed candidate | recursive graph node |
| Closure/Macro | Rc + captured Environment | managed candidate | participates in natural env cycles |
| Environment/Frame | Rc/RefCell | managed candidate | owns bindings and parent edges |
| Vector | Rc/RefCell | managed candidate | mutable aggregate; potential cycles |
| Symbol/String | Rc<str> | defer | immutable payload; no graph edges |
| Rational/BigInt | owned immutable payload | defer + measure | exact arithmetic pressure is separate from cycle collection |
| Host/resource handles | host-owned | explicit-resource | nondeterministic finalization must not own close semantics |

## Decision gate

The old Pair-only consensus is retained as historical design capital, but is no longer sufficient evidence for implementation. #153 must select the first managed slice from current cycle witnesses and migration cost.

The leading candidate is a **graph-core** slice:

```text
Pair + Closure + Environment + Vector
```

This is a candidate, not yet implementation authority. #154 must not move these types until #153 is ratified.

## Non-negotiable portability constraints

1. No raw Rust pointer is a semantic identity.
2. Managed references must admit stable backend-neutral handles; `slot + generation` remains the leading representation.
3. Host handles keep explicit lifecycle semantics; GC may diagnose/fallback-clean but cannot define language-visible close timing.
4. Collector on/off or stress frequency must not change Lisp value/output/error semantics.
5. The same reachability/object contract must be representable by native Rust, CML, and a future FPGA Lisp-machine.

## Implementation order retained from current architecture

```text
#153 current ownership/cycle evidence
  ↓
#154 ValueStorage facade (behavior-preserving)
  ↓
#155 ManagedHeap + ObjectId/generation
  ↓
#156 explicit roots + safe points
  ↓
#157 exact non-moving mark/sweep + stress/metamorphic proof
  ↓
#158 backend-neutral object contract
```

No collector code belongs in #153.