# Repo Tooling Inventory Design

**Issue:** #382  
**Status:** approved direction, implementation design  
**Date:** 2026-09-17

## Goal

Give `my-lisp` one Lisp-owned, machine-readable inventory of active repo-owned tooling so the repository can answer what each tool is, who calls it, what authority it depends on, its lifecycle state, and when it may be removed.

The inventory is governance metadata and a projection over repository state. It is **not** semantic authority and must not become a second source of language truth.

## Architectural choice

Use a dedicated Lisp registry under `knowledge/` plus a Lisp checker. Keep Rust/`xtask` semantics-blind: it may invoke the checker and surface pass/fail diagnostics, but it must not duplicate the inventory schema or own lifecycle policy.

This follows existing repository patterns such as `scripts/build-inventory.lisp`: Lisp owns structured meaning; generated/projection layers do not outrank their sources.

Rejected alternatives:

1. **Hand-maintained Markdown table** — easy to read but drift-prone and not a reliable machine surface.
2. **Rust-owned inventory/checker** — easy to wire into `xtask`, but would move repository policy data into the host and duplicate Lisp-owned governance.
3. **Guard as the registry itself** — rejected because Guard is navigation/reference infrastructure, not the owner of every tooling row.

## Files and responsibilities

### `knowledge/repo-tooling-inventory.lisp`

Owns one row per active repo-owned tooling entrypoint.

Each row records at least:

```text
path
kind
language
role
lifecycle
callers
authority-source
migration-issue
replacement
removal-condition
```

Closed vocabularies for the first slice:

```text
kind:
  check | generator | migration | benchmark | deploy | release | helper | other

lifecycle:
  active | transitional | legacy | generated-helper | archive-candidate |
  parity-green | switched-to-lisp | removable | bootstrap-exception

language:
  lisp | python | shell | javascript | powershell | rust | other
```

Unknown factual metadata must be represented explicitly as `unknown` or `()` according to field type. The registry must not invent callers, replacements, or authority.

A representative row shape:

```lisp
(tool
  (path "scripts/generate-meta-eval-evidence.py")
  (kind generator)
  (language python)
  (role meta-eval-human-evidence-projection)
  (lifecycle transitional)
  (callers ("crates/xtask/src/checks.rs"))
  (authority-source "knowledge/meta-eval-evidence.lisp")
  (migration-issue 351)
  (replacement ())
  (removal-condition parity-green-and-callers-switched))
```

`replacement` remains empty until the replacement path really exists. Planned filenames are not reported as repository facts.

### `scripts/check-repo-tooling-inventory.lisp`

Owns executable repository checks for this inventory.

First-slice checks:

- every active top-level `scripts/*` tooling entrypoint is represented exactly once;
- every registered live path exists;
- duplicate `path` rows fail;
- unknown `kind`, `language`, or `lifecycle` values fail closed;
- an active/transitional repo-owned Python tool has a migration issue or an explicit `bootstrap-exception` lifecycle;
- `bootstrap-exception` must carry an explicit removal condition;
- a `removable`/retired-style row cannot silently contradict live repository state;
- callers and authority-source paths that are declared as repository paths are checked for existence where mechanically distinguishable from issue IDs or symbolic provenance;
- tests/fixtures, vendored dependencies, archives, and immutable historical evidence are not automatically treated as active tooling.

The checker emits one small machine-readable verdict and precise diagnostics. It must not implement language semantics.

### `crates/xtask/src/checks.rs`

Add one thin governance check that invokes the Lisp checker through the canonical `my-lisp` runtime path and reports non-zero/negative verdicts.

Do not mirror the inventory rows or closed vocabularies in Rust.

### `knowledge/guard-reference.lisp`

Add one navigation topic, `repo-tooling`, that points to:

- `knowledge/repo-tooling-inventory.lisp`;
- `scripts/check-repo-tooling-inventory.lisp`;
- issue #382.

Guard does not copy the inventory rows.

## Coverage boundary

The first implementation covers active repo-owned tooling entrypoints under top-level `scripts/`.

It deliberately does not yet classify:

- every test helper under `scripts/tests/` as active tooling;
- GitHub workflows themselves — #384 owns workflow lifecycle;
- all `knowledge/*` artifacts — #383 owns authority classification there;
- environment-path requirements — #385 owns those;
- root artifacts — #23 owns root lifecycle;
- physical reorganization of `scripts/`;
- Python→Lisp implementation migration itself — #76 and child issues own that work.

The inventory may reference those tasks but must not absorb them.

## Gate rollout

### Phase 1 — RED witness

Create a focused negative fixture/witness proving that an intentionally unregistered active tool is detected.

### Phase 2 — current inventory GREEN

Populate rows for the current active top-level `scripts/*` surface until the checker passes on the exact branch head.

### Phase 3 — protection against new debt

Wire the checker into the cheap repository verification lane. Once current coverage is complete, a newly added active top-level tool without a row is a hard failure.

Historical uncertainty remains reportable rather than guessed into a lifecycle state.

## Python migration relationship

#382 does not create a second Python roadmap. For Python tools it records the state of #76 migration work.

Expected lifecycle interpretation:

```text
active/transitional + migration-issue -> migration is tracked
parity-green                        -> parity witness exists
switched-to-lisp                    -> active callers use Lisp replacement
removable                           -> deletion preconditions are satisfied
bootstrap-exception                 -> temporary exception with removal condition
```

If an active Python tool lacks a migration issue and is not a justified bootstrap exception, the checker reports it.

## Error handling

Fail closed for schema violations and contradictions that are fully mechanical:

- duplicate path;
- unsupported enum value;
- missing required field;
- registered active path missing from the repository;
- unregistered active top-level tool after full coverage is established;
- Python migration state without required issue/removal metadata.

Do not fail by guessing uncertain facts. Unknown caller/provenance information remains explicit and is surfaced for follow-up.

## Testing strategy

Use RED→GREEN in small slices:

1. focused negative witness for unregistered tooling;
2. duplicate-row witness;
3. missing-live-path witness;
4. invalid-enum witness;
5. Python-without-migration witness;
6. bootstrap-exception-without-removal-condition witness;
7. full current inventory coverage;
8. `cargo run -p xtask -- verify` integration after the standalone Lisp checker is green.

The tests must protect repository-governance behavior only; they must not introduce expected language semantics into Rust.

## Acceptance

The slice is complete when:

- every active top-level repo-owned `scripts/*` tool has one row;
- the inventory is Lisp-owned and machine-readable;
- unregistered new tooling is mechanically detectable;
- stale registered paths are mechanically detectable;
- active Python tooling is linked to #76 migration state or an explicit bootstrap exception;
- Guard can navigate to the inventory without copying it;
- `xtask verify` can run the check without owning the policy data;
- no scripts are moved and no Python tool is deleted merely to satisfy this issue.

## Principle

**The repository should remember why a tool exists before humans decide where it belongs or whether it can disappear.**
