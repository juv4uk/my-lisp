# `Value::Builtin` identity migration map — superseded status

This path is retained as a stable pointer because active plans and code comments reference it.

The original 2026-09-11 document described the **pre-implementation research baseline** in which Canon callables were still represented by Rust `Value::Builtin` objects and `Value::SemanticRef` had not yet been implemented. That historical text is now archived at:

`docs/archive/historical/BUILTIN-IDENTITY-MIGRATION-MAP-2026-09-11.md`

## Current state — 2026-09-13

PR #95 (`Canon SemanticRef: RED witness for semantic callable identity`) superseded the implementation-state claims in the original document.

Current observable/architectural facts:

- Canon callable identity is the numeric semantic ID from the language registry, not a Rust allocation/pointer.
- Canon callable values materialize as `Value::SemanticRef(id)`.
- admitted peer surfaces of one Canon semantic ID are `eq`; different semantic IDs remain distinct.
- evaluator application resolves a `SemanticRef` into the current implementation projection.
- `TAG_PRIMITIVE` carries the numeric semantic ID; host-only builtin pointers use a distinct host tag.
- unknown semantic callable IDs fail closed.
- legacy non-Canon `Value::Builtin` values may remain as host mechanisms; their Rust identity is not Canon semantic identity.

The archived research map remains useful as provenance for **why** pointer identity was rejected and which migration seams were originally identified. It is non-normative and must not override current Canon, `semantic-registry.wsm`, language contracts, or executable tests.
