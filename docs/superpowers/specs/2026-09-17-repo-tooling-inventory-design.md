# Repo Tooling Inventory Design

**Issue:** #382  
**Status:** approved implementation design  
**Date:** 2026-09-17

## Goal

Give `my-lisp` one Lisp-owned, machine-readable inventory of active repo-owned tooling so the repository can answer what each tool is, who calls it, what authority it depends on, its repository lifecycle, and when it may be removed.

The inventory is governance metadata and a projection over repository state. It is **not** semantic authority and must not become a second source of language truth.

> **The repository should remember why a tool exists before humans decide where it belongs or whether it can disappear.**

## Global constraints

Issue #299 is authoritative while host retirement is active:

```text
Lisp may grow.
Existing Rust may remain unchanged where still necessary.
Rust may only shrink.
```

Therefore #382 adds **zero Rust lines** and no new `.rs` files. #76 remains the only Python→Lisp migration authority. #382 records migration ownership; it does not create a second roadmap. No script is moved or deleted merely to make this inventory green.

## Architectural choice

Use a dedicated Lisp registry under `knowledge/` plus a Lisp checker under `scripts/`. The steady-state gate runs that checker through the existing `my-lisp` CLI from the existing CI workflow. Do not create a second policy implementation in Rust, Python, Markdown, or a permanent new workflow.

A temporary RED-only workflow may be used to isolate a failing witness while the ordinary CI lane is blocked by unrelated baseline work, but it is verification scaffolding only and **must be removed before the PR is ready**. Workflow lifecycle belongs to #384.

## Files and responsibilities

### `knowledge/repo-tooling-inventory.lisp`

Owns one row per in-scope repo-owned tooling entrypoint.

Each row records:

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
  active | transitional | legacy | generated-helper | archive-candidate

language:
  lisp | python | shell | javascript | powershell | other
```

Unknown factual metadata is represented explicitly as `unknown` or `()` according to field type. The registry must not invent callers, replacements, or authority.

Representative row:

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

`replacement` remains empty until the replacement path really exists. Planned filenames are not repository facts.

## Lifecycle and migration state are different dimensions

`lifecycle` answers:

> What is this artifact's repository role/age?

Migration progress answers:

> Where is this tool in a replacement process?

The first slice deliberately keeps only artifact `lifecycle` plus `migration-issue`. Python migration progress remains owned by #76 and must **not** be encoded into lifecycle values such as `parity-green`, `switched-to-lisp`, `removable`, or `bootstrap-exception`.

If the mature inventory needs to express both dimensions simultaneously — for example `archive-candidate` **and** `parity-green` — that is the explicit schema-evolution trigger to split into:

```text
lifecycle
migration-state
```

Any future `migration-state` is only a projection/index of #76, never an independent migration authority.

A temporary bootstrap exception, if one exists, is represented through explicit migration/removal metadata rather than masquerading as an artifact lifecycle.

### `scripts/check-repo-tooling-inventory.lisp`

Owns executable governance checks. Its core validation is pure Lisp over:

```text
inventory rows + observed immediate script-entry names
```

The filesystem-backed runner only obtains real observations (`read-dir`, `read-file`) and feeds them into the validator. Negative witnesses therefore do not need to mutate the repository.

First-slice mechanical checks:

- every in-scope immediate `scripts/*` entry is represented exactly once;
- every registered in-scope live path exists in observed `scripts/` entries;
- duplicate `path` rows fail;
- missing required fields fail;
- unknown `kind`, `language`, or `lifecycle` values fail closed;
- active/transitional repo-owned Python has a `migration-issue` pointing to #76 or a child issue;
- declared replacement path is not presented as real before it exists;
- explicit removal conditions are preserved rather than inferred.

The checker emits a small machine-readable verdict plus precise diagnostics. It does not implement language semantics and must use current explicit result domains rather than generic `t/()` truth authority.

### Existing CI workflow

The final gate is one neighboring step in `.github/workflows/ci.yml`, executed through the canonical CLI. No Rust glue and no permanent new workflow.

### `knowledge/guard-reference.lisp`

Add one navigation topic `repo-tooling` pointing to the inventory, checker, design, and #382. Guard does not copy individual tooling rows.

## Coverage boundary

The first implementation covers immediate entries returned by `read-dir("scripts")`, minus the explicit out-of-scope directory entry `tests`.

It deliberately does not classify:

- nested `scripts/tests/*` helpers;
- GitHub workflow lifecycle (#384);
- all `knowledge/*` authority classes (#383);
- environment paths (#385);
- root artifact lifecycle (#23);
- physical reorganization of `scripts/`;
- Python→Lisp implementation migration itself (#76 and child issues).

## RED → GREEN discipline

A RED counts only when it reaches the behavior under test. Parser errors, stale-branch failures, or an unrelated authority guard are **not** valid RED evidence.

Required progression:

1. isolate one missing governance rule with an executable witness;
2. observe the exact expected failure on the synchronized branch;
3. implement the smallest Lisp change that makes that witness pass;
4. keep previously-green witnesses green;
5. repeat for duplicate path, stale path, invalid enum, and Python migration ownership;
6. populate the real inventory and enable final filesystem-backed enforcement;
7. remove any temporary RED-only workflow before completion.

## Anti-gaming rules

- No script is moved merely to leave the checker's scope.
- No Python file is deleted merely to make inventory coverage green.
- Unknown caller/authority facts stay explicit instead of being guessed.
- Generated artifacts point to generator/upstream authority rather than acquiring authority from the inventory.
- Replacement remains `()` until the file exists.
- The checker never becomes a semantic oracle.

## Acceptance

The slice is complete when:

- every in-scope immediate repo-owned `scripts/*` entry has exactly one row;
- the inventory is Lisp-owned and machine-readable;
- unregistered new tooling and stale registered paths are mechanically detectable;
- duplicate rows and invalid schema enums are mechanically detectable;
- active/transitional Python tooling is linked to #76 or a narrower child issue without inventing a second migration state machine;
- Guard navigates to the inventory without copying it;
- the existing CI lane runs the checker through `my-lisp` CLI;
- temporary RED-only workflow scaffolding is gone;
- zero Rust lines/files are added;
- no scripts are moved and no Python tool is deleted merely to satisfy #382.

## Principle

**The repository should remember why a tool exists before humans decide where it belongs or whether it can disappear.**
