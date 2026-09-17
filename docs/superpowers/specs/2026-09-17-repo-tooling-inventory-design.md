# Repo Tooling Inventory Design

**Issue:** #382  
**Status:** approved direction, implementation design  
**Date:** 2026-09-17

## Goal

Give `my-lisp` one Lisp-owned, machine-readable inventory of active repo-owned tooling so the repository can answer what each tool is, who calls it, what authority it depends on, its lifecycle state, and when it may be removed.

The inventory is governance metadata and a projection over repository state. It is **not** semantic authority and must not become a second source of language truth.

## Global constraint: Rust retirement valve

Issue #299 is authoritative for this work while host retirement is active:

```text
Lisp may grow.
Existing Rust may remain unchanged where still necessary.
Rust may only shrink.
```

Therefore #382 must add **zero Rust lines** and no new `.rs` files. In particular, it must not add an `xtask` check even though `xtask` is otherwise a natural governance surface. The inventory/checker and their CI integration must be non-Rust.

## Architectural choice

Use a dedicated Lisp registry under `knowledge/` plus a Lisp checker under `scripts/`. Run that checker through the existing `my-lisp` CLI from an existing CI workflow; do not create a second policy implementation in Rust, Python, or Markdown.

This follows existing repository patterns such as `scripts/build-inventory.lisp`: Lisp owns structured meaning; projections and host mechanisms do not outrank their sources.

Rejected alternatives:

1. **Hand-maintained Markdown table** — easy to read but drift-prone and not a reliable machine surface.
2. **Rust/`xtask`-owned inventory or verifier** — conflicts with #299 and would move repository policy data into the host.
3. **Python checker** — conflicts with #76 and creates fresh Python debt while Python is being removed.
4. **Guard as the registry itself** — rejected because Guard is navigation/reference infrastructure, not the owner of every tooling row.
5. **New dedicated workflow** — unnecessary; #384 already owns workflow lifecycle and #382 only needs one extra step in an existing cheap CI lane.

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
  lisp | python | shell | javascript | powershell | other
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

Owns executable repository checks for this inventory and may use the existing CLI-installed filesystem capabilities (`read-dir`, `read-file`, etc.).

The mechanical coverage domain for the first slice is exact:

```text
all immediate entries returned by read-dir("scripts")
minus the explicit out-of-scope directory entry "tests"
```

A new immediate entry under `scripts/` is therefore fail-closed until it is either inventoried as tooling or the scope rule is deliberately amended. Nested `scripts/tests/*` helpers are left to a later slice and are not silently guessed to be production tooling.

First-slice checks:

- every in-scope immediate `scripts/*` entry is represented exactly once;
- every registered in-scope live path exists in the observed `scripts/` entries;
- duplicate `path` rows fail;
- unknown `kind`, `language`, or `lifecycle` values fail closed;
- an active/transitional repo-owned Python tool has a migration issue or an explicit `bootstrap-exception` lifecycle;
- `bootstrap-exception` must carry an explicit removal condition;
- a `removable`/retired-style row cannot silently contradict live repository state;
- declared repository-relative caller/authority paths are checked for existence only where the checker can do so without guessing their meaning;
- vendored dependencies, archives, immutable historical evidence, and `scripts/tests/*` are not automatically treated as active tooling.

The checker emits a small machine-readable final verdict and precise human diagnostics. It must not implement language semantics.

### Existing CI workflow

Add one step to an existing cheap CI lane, using the canonical CLI execution pattern, for example:

```sh
cargo run -p my-lisp-cli --bin my-lisp -- scripts/check-repo-tooling-inventory.lisp
```

Do not create a new workflow solely for #382. Do not add Rust glue. Workflow lifecycle/placement remains reviewable under #384.

### `knowledge/guard-reference.lisp`

Add one navigation topic, `repo-tooling`, that points to:

- `knowledge/repo-tooling-inventory.lisp`;
- `scripts/check-repo-tooling-inventory.lisp`;
- issue #382.

Guard does not copy the inventory rows.

## Coverage boundary

The first implementation covers active repo-owned tooling entrypoints that are immediate entries under top-level `scripts/`, except the explicitly excluded `scripts/tests` directory.

It deliberately does not yet classify:

- every test helper under `scripts/tests/` as active tooling;
- GitHub workflows themselves — #384 owns workflow lifecycle;
- all `knowledge/*` artifacts — #383 owns authority classification there;
- environment-path requirements — #385 owns those;
- root artifacts — #23 owns root lifecycle;
- physical reorganization of `scripts/`;
- Python→Lisp implementation migration itself — #76 and child issues own that work;
- any Rust additions — forbidden by #299.

The inventory may reference those tasks but must not absorb them.

## Gate rollout

### Phase 1 — RED witness

Create focused Lisp-owned negative fixtures/witnesses for schema/policy behavior, including an intentionally unregistered observed tool.

The checker should expose its core validation as a pure function over:

```text
inventory rows + observed script entry names
```

so negative cases do not need to mutate the real working tree.

### Phase 2 — current inventory GREEN

Populate rows for the current in-scope immediate `scripts/*` surface until the checker passes against real `read-dir("scripts")` observations on the exact branch head.

### Phase 3 — protection against new debt

Wire the standalone Lisp checker into an existing cheap CI lane. Once current coverage is complete, a newly added immediate `scripts/*` entry without a row is a hard failure.

Historical uncertainty remains explicit/reportable rather than guessed into a lifecycle state.

## Python migration relationship

#382 does not create a second Python roadmap. For Python tools it records the state of #76 migration work.

Expected lifecycle interpretation:

```text
active/transitional + migration-issue -> migration is tracked
parity-green                          -> parity witness exists
switched-to-lisp                      -> active callers use Lisp replacement
removable                             -> deletion preconditions are satisfied
bootstrap-exception                   -> temporary exception with removal condition
```

If an active Python tool lacks a migration issue and is not a justified bootstrap exception, the checker reports it.

## Error handling

Fail closed for schema violations and contradictions that are fully mechanical:

- duplicate path;
- unsupported enum value;
- missing required field;
- registered in-scope path absent from observed immediate `scripts/*` entries;
- unregistered observed immediate `scripts/*` entry;
- Python migration state without required issue/removal metadata.

Do not fail by guessing uncertain facts. Unknown caller/provenance information remains explicit and is surfaced for follow-up.

## Testing strategy

Use RED→GREEN in small Lisp-owned slices:

1. focused negative witness for unregistered tooling;
2. duplicate-row witness;
3. stale/missing-path witness;
4. invalid-enum witness;
5. Python-without-migration witness;
6. bootstrap-exception-without-removal-condition witness;
7. full current inventory coverage against `read-dir("scripts")`;
8. existing-CI integration through the CLI;
9. authority/Rust valve checks proving the change adds zero Rust lines.

The tests protect repository-governance behavior only; they must not introduce expected language semantics into Rust.

## Acceptance

The slice is complete when:

- every in-scope immediate repo-owned `scripts/*` entry has one row;
- the inventory is Lisp-owned and machine-readable;
- unregistered new tooling is mechanically detectable;
- stale registered in-scope paths are mechanically detectable;
- active Python tooling is linked to #76 migration state or an explicit bootstrap exception;
- Guard can navigate to the inventory without copying it;
- an existing CI lane runs the checker through `my-lisp` CLI;
- zero Rust lines/files are added, preserving #299;
- no scripts are moved and no Python tool is deleted merely to satisfy this issue.

## Principle

**The repository should remember why a tool exists before humans decide where it belongs or whether it can disappear.**
