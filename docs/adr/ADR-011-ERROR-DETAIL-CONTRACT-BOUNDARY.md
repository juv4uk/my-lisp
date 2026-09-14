# ADR-011 — Error detail contract boundary

## Українське резюме

Поточний cross-runtime контракт помилки визначається **категорією**, а не текстом повідомлення, span чи implementation-specific payload. Цей ADR тепер явно називає чинне правило як **Error Observation Identity v1**: одна admitted error category = одна semantic error identity для conformance. Поля на кшталт `x` у `(error unbound-symbol x)` лишаються діагностичними, доки окремий ADR не ратифікує їх через machine-readable schema та спільні executable witnesses.

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

## Error Observation Identity v1

This section names the decision above; it does **not** expand it.

For cross-runtime conformance today:

```text
semantic error identity v1
          =
admitted error category
```

The category is the identity-relevant observation. Message text, source span,
meta-evaluator `detail`, implementation-internal exception/control flow, and
payload spellings are diagnostic or implementation data unless a later
contract explicitly promotes them.

Therefore two meta-level values such as:

```lisp
(error unbound-symbol x)
(error unbound-symbol y)
```

belong to the same current cross-runtime semantic error class when both map to
the same admitted category. The differing `x`/`y` payload remains useful
information, but it is not part of Error Observation Identity v1.

This naming is important for backend-neutral witnesses: an implementation must
not be rejected merely because its diagnostic payload differs in a field the
language has never ratified as semantic; conversely, it must not invent a new
observable category and hide that change inside a payload.

In short:

```text
language semantics      diagnostic surfaces
------------------      -------------------
error category          Rust message
                        Rust source span
                        meta Lisp detail payload
                        implementation-local payload spellings
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
- No localization text is copied into `lib/meta-eval.lisp`.
- No source-span equivalence is required between host evaluator and metacircular evaluator.
- Useful structured Lisp diagnostics are preserved.
- `error-detail-parity` can be confirmed only as an **explicit boundary result**: there are currently no additional ratified cross-runtime detail fields beyond the already-confirmed category.
- Error Observation Identity v1 remains category-only; naming it does not authorize new payload equality requirements.
- This does **not** by itself authorize the project phrase `complete self-hosting`; claim vocabulary remains separately governed by the evidence matrix and project policy.

## Future extension rule

If a consumer genuinely needs stable machine-readable details — for example `expected=2, received=1` or a host operation/reason pair — that field may become identity-relevant only after all three gates are satisfied:

1. an explicit contract/ADR decision names the field as semantic rather than diagnostic;
2. an implementation-neutral, machine-readable schema defines its representation;
3. shared executable witnesses require the field across implementations.

Until then, adding a Rust `ErrorDetail` enum or promoting a meta-evaluator payload solely to make current tests look more alike is rejected as unnecessary mechanism growth and an accidental contract expansion.
