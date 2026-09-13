# Full Ukrainian Cyrillic-Only Surface Design

## Purpose

Make `full-uk` a first-class peer surface projection of the existing Canon semantic identities while preserving the current `uk` surface for compatibility.

The human goal is operational, not merely linguistic: a programmer who chooses `full-uk` must be able to type a normal my-lisp program while leaving the keyboard in the Ukrainian layout. The surface therefore must not require Latin letters inside public identifiers.

## Identity model

`semantic ID` remains the only semantic identity. `uk` and `full-uk` are two spellings/projections of the same identity:

```text
semantic ID
   ├─ uk       compatibility/current Ukrainian spelling
   ├─ full-uk  full-word Ukrainian spelling
   ├─ en       English spelling
   ├─ sa       Sanskrit spelling
   └─ sym      language-neutral notation
```

Adding or changing `full-uk` must never create a new semantic implementation, duplicate a function, or make a display string the identity.

## Authority

`lib/surface/semantic-registry.wsm` remains the single spelling authority.

The staging file `lib/surface/український-профіль-джерела.всм` remains a review/workbench source only. A name is executable as `full-uk` only after it is represented in the semantic registry with an admitted status.

`lib/generated/function-table.wsm` and `docs/generated/function-table.md` remain generated projections and must not become a second source of truth.

## Cyrillic-only identifier rule

For an admitted `full-uk` identifier:

- alphabetic characters must be Ukrainian Cyrillic;
- Latin ASCII letters `A-Z` / `a-z` are forbidden;
- decimal digits are allowed;
- normal Lisp identifier punctuation used by the language is allowed, including `-`, `?`, `!`, and apostrophe where the reader admits it;
- the rule applies to public `full-uk` identifiers, not to string data, comments, file contents, foreign protocol payloads, paths, or arbitrary user data;
- no Latin technical allowlist exists for `full-uk`.

The practical acceptance criterion is: a representative program using the `full-uk` surface can be typed without switching away from the Ukrainian keyboard layout.

## Technical terminology

A foreign technical acronym must not force a Latin keyboard switch in `full-uk`.

Prefer a natural Ukrainian behavioral name when the protocol/format detail is not the semantic essence of the operation. If the specific standard is essential to the API contract, use a Ukrainian-script name chosen deliberately rather than silently retaining Latin letters.

Examples of direction, subject to behavioral review:

- `read-tcp`-style functionality → `прочитати-з-мережевого-з'єднання` rather than `прочитати-з-tcp`;
- `write-tcp`-style functionality → `записати-у-мережеве-з'єднання`;
- `listen-tcp`-style functionality → `слухати-мережеві-з'єднання`;
- JSON/SHA-specific compatibility helpers must receive a deliberate Cyrillic spelling or remain non-admitted in `full-uk` until reviewed.

## Admission and compatibility

Current `uk` spellings remain intact. `full-uk` may equal `uk` when the existing spelling is already a good full-word Ukrainian name.

Statuses retain existing semantics:

- `stable` — admitted and executable;
- `candidate` — recorded but not admitted;
- `missing` — no spelling;
- `compatibility-only` — admitted only when intentionally preserving a legacy spelling.

`full-uk` compatibility candidates do not automatically create a stable public spelling. Duplicate `full-uk` spellings across different semantic identities are forbidden unless the repository's explicit alias mechanism proves that the rows represent the same intended owner relationship.

## Runtime behavior

The native semantic registry parser is namespace-generic. Therefore a `(full-uk NAME stable)` triple is resolved through the same numeric semantic-ID machinery as `uk`, `en`, `sa`, or `sym` and does not require a separate evaluator implementation.

The meta-evaluator generated registry must likewise include admitted `full-uk` spellings because it projects admitted namespaces generically from `semantic-registry.wsm`.

## Generator behavior

`scripts/generate-function-table.my` must stop synthesizing `full-uk` by mirroring `uk` once `full-uk` becomes authoritative. It must read the `full-uk` surface directly from each registry row and emit that value/status into generated machine and Markdown tables.

Rows that have not yet been ratified may explicitly carry `(full-uk — missing)` or a candidate spelling. The generator must never invent a name.

## Guards

CI/tests must prove at least these properties:

1. every admitted `full-uk` spelling maps to the same semantic ID as its peer surfaces;
2. admitted `full-uk` spellings contain no ASCII Latin letters;
3. adding `full-uk` never removes or changes an existing stable `uk` spelling;
4. duplicate admitted spellings across different semantic identities fail closed;
5. generated function tables reproduce registry `full-uk` values instead of mirroring `uk`;
6. a representative `full-uk` `.lisp` program executes successfully through the real CLI.

## First ratification slice

The first implementation slice should establish the authority mechanism and ratify only names that are semantically clear enough to defend. Existing `uk` names that already satisfy the full-word Cyrillic rule can be copied as `full-uk stable` without changing `uk`.

The six stable host API identities currently lacking a Ukrainian surface must be reviewed under the no-layout-switch rule before admission. Latin-containing staging proposals such as `прочитати-з-tcp`, `розібрати-json`, and `sha256-у-шістнадцятковий-текст` are specifically not acceptable as `full-uk stable` in their current spelling.

## Non-goals

- translating internal Rust identifiers;
- banning Latin text in strings/data/comments;
- forcing Cyrillic filenames;
- renaming or removing existing stable `uk` spellings;
- inventing a second Ukrainian implementation tree;
- changing semantic IDs or Canon meaning.

## Completion criterion

The feature is complete when the semantic registry is the authority for `full-uk`, admitted full-Ukrainian identifiers are Cyrillic-only by invariant, the generated tables preserve the authoritative names, and an executable acceptance program demonstrates normal source code typed without switching to a Latin keyboard layout.