# ADR-011 — Error detail contract boundary

Status: accepted for the current meta-evaluator evidence scope · 2026-09-10

## Context

Issue #26 requires `error-detail-parity` **where observable**, but it also says that intentionally non-contractual details must be named explicitly rather than silently discarded by normalization.

The existing language rule already draws that boundary. S2 says every failure has a named observable category and states that wording may differ: **the category is the contract**. `LanguageError` accordingly stores `kind`, `message`, and `span`, while `ErrorKind` is the contractual axis and presentation/classification data may evolve independently.

The Lisp meta-evaluator exposes failures as Lisp data of the form:

```lisp
(error kind detail)
```

Its `detail` is useful diagnostic data. Examples include exact/minimum arity counts and malformed lambda-list structure. The Rust reference evaluator, however, does not expose a ratified structured field corresponding to that payload; it exposes human-oriented `message` plus source `span`.

Treating Rust prose or byte offsets as language semantics would make self-hosting parity depend on implementation wording, localization, and source layout. Conversely, deleting the meta-evaluator's structured diagnostic payload merely to make outputs look alike would throw away useful Lisp-owned information.

## Decision

For the current language contract and the meta-evaluator evidence matrix:

1. **The named error category is the cross-runtime contractual observation.** Reference `ErrorKind` is compared with the corresponding meta-evaluator kind (`UnknownSymbol ↔ unbound-symbol`, `Type ↔ not-callable`, `Arity ↔ arity`, `InvalidForm ↔ invalid-form`).
2. **`LanguageError.message` is diagnostic/presentation text, not a cross-runtime semantic field.** Differential tests must not parse or copy its prose to manufacture parity.
3. **`LanguageError.span` is source-location metadata, not a cross-runtime semantic field.** Different surface spellings and encodings may naturally produce different byte ranges while denoting the same failure category.
4. **The meta-evaluator `detail` payload remains available as Lisp diagnostic data, but no cross-runtime detail schema is ratified today.** Exact shapes may have implementation-level regression tests without becoming a language-conformance requirement.
5. A future contract may ratify a structured detail schema. If that happens, it must be explicit, machine-readable, and shared by implementations. Until then, adding a Rust `ErrorDetail` enum solely to satisfy the current evidence row is rejected as unnecessary mechanism growth.

In short:

```text
language semantics      diagnostic surfaces
------------------      -------------------
error category          Rust message
                        Rust source span
                        meta Lisp detail payload
```

The right side may be tested for local quality. It is not silently normalized into a fake cross-runtime equality claim.

## Executable evidence

`crates/my-lisp/tests/meta_eval_error_detail_boundary.rs` guards this boundary by checking that:

- the S2 wording continues to state category-over-wording semantics;
- the Rust implementation continues to document `kind` as the contractual axis;
- equivalent Canon-rebinding failures can carry different source spans while retaining one `InvalidForm` category;
- meta-eval structured arity/lambda details remain present rather than being erased.

The existing paired error tests continue to prove category correspondence. Their manually normalized Lisp detail expectations are regression witnesses for the meta representation, not assertions that Rust human prose has the same structure.

## Consequences

- No parser for native error messages is introduced.
- No localization text is copied into `lib/meta-eval.my`.
- No source-span equivalence is required between host evaluator and metacircular evaluator.
- Useful structured Lisp diagnostics are preserved.
- `error-detail-parity` can be confirmed only as an **explicit boundary result**: there are currently no additional ratified cross-runtime detail fields beyond the already-confirmed category.
- This does **not** by itself authorize the project phrase `complete self-hosting`; claim vocabulary remains separately governed by the evidence matrix and project policy.

## Future extension rule

If a consumer genuinely needs stable machine-readable details — for example `expected=2, received=1` or a host operation/reason pair — first add an explicit contract/ADR and an implementation-neutral representation. Only then may such a field become part of cross-runtime parity.
