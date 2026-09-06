# my-lisp semantic authority map

Status: CURRENT ARCHITECTURE MAP. This file does not create new language semantics. It defines where an existing claim must be checked before prose is trusted.

## One rule

> The language defines its semantics; runtimes prove conformance to it.
>
> Rust knows mechanisms. Lisp knows the language.

No implementation file, README paragraph, agent note, benchmark, or historical plan may silently outrank the semantic contract.

## Authority order

When two sources disagree, use this order:

1. **`language-contract.my`** — machine-readable Level 1/2 contract version and ratified observable invariants.
2. **Ratified ADRs under `docs/adr/`** — closed decisions whose scope is explicitly stated, especially `ADR-004-CLOSED-MCCARTHY7-CORE.md`.
3. **Executable conformance evidence** — `tests/fixtures/conformance.my`, `tests/fixtures/macro-conformance.my`, `lib/canon.my`, and the tests that execute those contracts.
4. **Reference implementation** — `crates/my-lisp`. It is the mature software oracle used to test behavior, not the owner of semantics merely because it is Rust.
5. **Independent implementations** — `fpga-lisp`, `c-runtime/`, and other declared substrates. Their value is that they can falsify implementation-specific assumptions.
6. **Generated reference** — for example `docs/FUNCTIONS.md`. Generated output describes the current implementation surface but does not redefine the contract.
7. **Human explanatory prose** — `README.md`, `docs/language-core.md`, tutorials, architecture notes.
8. **Historical/process material** — `PLAN.md`, dated audits, agent notes, superseded decisions. These are evidence of history, not current semantic authority.

If a lower item conflicts with a higher item, the lower item is stale until reconciled.

## Semantic identity vs surface spelling

A spelling is not a primitive identity. The closed semantic set remains exactly seven operations:

```text
quote · atom · eq · cons · car · cdr · cond
```

The project also treats the concrete empty proper list `()` as Canon 0: a value/syntax identity, not an eighth operation and not a lexical alias.

Natural-language and historical surfaces map onto semantic identities; changing a surface name does not create a new primitive.

## Bootstrap forms

Do not collapse three different questions:

- **semantic primitives** — permanently closed 0+7 model above;
- **evaluator-controlled bootstrap forms** — machinery needed to create language behavior;
- **derived language forms** — behavior expressible by the language once the bootstrap substrate exists.

The current implementation is actively reducing host-owned bootstrap behavior. `lib/macro.my` owns the normal `defmacro` binding after the macro layer loads; a Rust fallback still exists during migration. `define`/`lambda` have explicit necessary-form identities in the evaluator, while historical `def` compatibility remains. These are implementation facts and do not silently rewrite `language-contract.my`; observable-contract changes still require the contract process.

## Project identity and source extensions

The project/repository name is **`my-lisp`**.

The current canonical source extension is **`.wsm`**. **`.my`** and **`.lisp`** remain fully supported aliases. The separate repository named `wsm` is unrelated foundational research; the shared letters do not rename this language project.

## Reference implementation terminology

Use these terms consistently:

```text
semantic authority      = contract + ratified decisions + executable conformance
reference implementation = crates/my-lisp (Rust)
independent substrate    = fpga-lisp / c-runtime / other conformance target
```

Avoid “canonical Rust implementation” when the intended meaning is “reference implementation”. A canonical implementation would imply that Rust itself defines semantics, which contradicts the conformance architecture.

## Host boundary

A host operation earns its place by providing information or effects unavailable inside pure language semantics. A useful test is:

> Does this operation reveal a new fact about the external world, or does it merely transform facts the language already has?

Examples:

```text
mono-ns        -> host observation
mono-ms        -> Lisp derivation
unix-time-now  -> host observation
UTC/Gregorian  -> Lisp interpretation
filesystem I/O -> host capability
path/policy     -> Lisp whenever derivable
```

See `docs/host-semantic-surface.md` for the living inventory.

## Documentation rule

New prose that states a contract-level fact should link to the authoritative source instead of re-specifying the fact in a new independent wording. If repetition is necessary for teaching, phrase it as a summary and keep a drift test for facts that are easy to contradict mechanically.

The goal is not fewer documents at any cost. The goal is one authority for each kind of claim.
