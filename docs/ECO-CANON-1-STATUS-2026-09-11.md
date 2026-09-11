# ECO-CANON-1 status — 2026-09-11

Issue: [my-lisp#75](https://github.com/juv4uk/my-lisp/issues/75)

## Principle (unchanged)

```text
CANON = immutable identity / meaning
display spelling ≠ identity
lib/surface/semantic-registry.wsm = sole surface↔ID authority
```

## What already existed

| Piece | Role |
|-------|------|
| `lib/surface/semantic-registry.wsm` | Numeric semantic IDs + en/uk/sa/sym surfaces |
| `scripts/check_semantic_registry.py` | Fail-closed schema checks |
| `scripts/generate-meta-semantic-registry.py` | Runtime projection for meta-eval |
| `crates/my-lisp/src/semantic_registry.rs` + `eval/canon.rs` | In-process resolvers |
| `docs/CANON-MIGRATION-PLAN-2026-09-11.md` | Consumer generator pattern |

## First deliverable this pass

1. **`scripts/generate-function-table.py`** — projects registry → function table (`ft/1`)
2. **`lib/generated/function-table.wsm`** — machine-readable table (161 identities)
3. **`docs/generated/function-table.md`** — human view, column order:
   **Українська → Повна українська → English → Sanskrit**

### Column policy (honest)

| Column | Source |
|--------|--------|
| `uk` | registry `uk` surface as-is |
| `full-uk` | **mirrors** stable/candidate `uk` until a separate ratification pass; **does not invent** names |
| `full-uk-status` | `stable` / `candidate` / `needs-review` / `missing` |
| `en` / `sa` / `sym` | registry |
| `authority` | always `my-lisp` for language identities |

At generation: **161** identities; **21** `full-uk` marked `needs-review`.

### Not done yet (remaining #75)

- Formal semantic *action* prose per ID (still stub)
- Executable witness column per row
- CI job with `generate-function-table.py --check`
- Distinct ratification of «Повна українська» where it should differ from `uk`
- Consumer-repo projections (must not hand-copy)

## Commands

```bash
python3 scripts/generate-function-table.py
python3 scripts/generate-function-table.py --check
python3 scripts/check_semantic_registry.py
```

## Consumer rule

Other repos **must not** hand-copy this table. Generate from `semantic-registry.wsm` or pin a published projection; unknown ID → fail-closed.
