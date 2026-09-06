# Repository audit resolution — 2026-09-07

**Status:** current repair ledger for the repository-wide audit performed on
2026-09-07. This is evidence/process documentation, not a new language
contract. Semantic authority remains `language-contract.my`, ratified ADRs and
executable conformance evidence.

The purpose of this file is to distinguish three states that must not be mixed:

- **fixed + confirmed** — code changed and an executable regression proves the
  reported failure no longer occurs on the tested boundary;
- **guarded / partial** — the dangerous path is now protected, but a broader
  architectural property is deliberately not claimed;
- **migration gate** — changing it immediately would itself be a compatibility
  or architecture decision, so it is recorded with an explicit removal/
  ratification condition rather than falsely labeled fixed.

## Resolution matrix

| Audit finding | Resolution | Evidence | Status |
|---|---|---|---|
| `reason` full rule scan used non-tail `append(... recursive scan ...)`, overflowing ordinary stack in the measured worst case | `prove-goal` now delegates to a tail-recursive scan, accumulates with `cons`, reverses once | `reason_stack.rs`: 256-rule default-stack full scan + result-order regression | **fixed + confirmed** |
| Scale harness still encoded the historical 64 MiB workaround after the stack repair | N=100/500/1000 now runs on the ordinary stack; fixture construction is non-recursive Rust-generated quoted data; 5k/10k is an ignored manual profile | `reason_scale.rs`, `reason-scale-profile-2026-08-29.md`, CI #1034 | **fixed + confirmed on refreshed harness** |
| `reason`/`reason-in` collapsed multiple epistemic meanings into ordinary `()` | One canonical opt-in data algebra in `result-status.my`: `proved / unknown / partial / blocked / disputed / invalid`; compatibility APIs unchanged | `result_status.rs`, `reason_outcome_invalid.rs`, `reason_in_outcome_invalid.rs` | **fixed + confirmed on tested outcome boundary** |
| Proposed B1 vocabulary duplicated the older `unknown/partial/blocked/disputed` convention | Existing convention was extended instead of creating a second algebra; ADR updated | `docs/adr/unknown-result-semantics.md` | **resolved** |
| A successful query may have several answers; a new wrapper could accidentally keep only the first | `proved` stores the complete `reason` result list; dispute stores evidence for both sides | multiple-alternative test in `result_status.rs` | **fixed + confirmed** |
| Malformed goals could be mislabeled `unknown` | Goal observation now requires a proper list + symbolic predicate; `(not goal)` has exact recursive shape; module/goal validation precedes module lookup | malformed `not`, non-symbol head, invalid module and missing-module precedence tests | **fixed + confirmed on tested shapes** |
| `narrate` explained proofs but collapsed failure modes | `narrate-outcome` preserves explicit status in presentation for all six outcome classes | `narrate_outcomes.rs` | **fixed + confirmed** |
| Malformed tagged presentation data could crash/access missing fields | `narrate-outcome` validates proper list, tag type and exact tag arity before field access | truncated/unknown/non-symbol-tag adversarial tests; CI #1030 | **fixed + confirmed** |
| Advice Taker lacked one adversarial end-to-end corpus over the new outcome path | Added 7-case corpus: direct, multi-step, recursive, unknown, conflict rejection, malformed advice, knowledge-package round-trip | `advice_corpus.rs`; CI #1020 | **fixed + confirmed** |
| External/NL translator could become a second semantic/write authority if wired directly to knowledge | Added versioned `translation/1` data protocol; Lisp-owned pure `translation-review`; only accepted clause/batch exposes an admission payload and actual write remains explicit `advise`/`advise-all`; rejected/ambiguous reviews remain evidence | `translation_boundary.rs`, `translation-corpus-v1.wsm`; CI #1055; Chrome + Firefox on same head | **first B4 boundary fixed + confirmed** |
| Committed `core.my.fasl` could drift from current `core.my` without a repository-wide invariant | Added exact embedded-source-hash regression | `core_fasl_freshness.rs` | **fixed + confirmed** |
| Browser/WASM suite did not run when semantic dependencies changed | Workflow path filters now include core runtime/literate/LSP/core source/FASL/Cargo graph | repeated WASM browser runs after semantic changes, Chrome + Firefox | **fixed + confirmed** |
| `wsm-guard-core` accepted rendered substring `"(decision allow)"` rather than a real protocol value | Rust adapter validates exact `guard-finding`, `guard/1`, ordered fields, decision enum and evidence-status enum before rendering | spoofed nested decision + wrong schema tests | **fixed + confirmed** |
| Filesystem/TCP host capabilities were all-or-nothing once installed | Added per-session read roots, write roots, TCP connect ranges and TCP listen ranges while preserving trusted `None = unrestricted`; enforcement stays in `my-lisp-host` | outside-root, `load` bypass, symlink escape, connect/listen deny-before-network tests; CI #1038 | **embedding mechanism fixed + confirmed on tested boundary** |
| `docs/FUNCTIONS.md` drifted after reasoning/narration/translation definitions | Inventory refreshed; documentation regression counts live `(def ...)` names and checks `result-status.my`, `narrate.my`, `translation.my` sections | `documentation_contract.rs`; CI #1055 | **fixed; guarded against recurrence on these modules** |
| Old reason scale report still read like the current broken state after repair | Report split into historical measurement vs current repaired harness/follow-up | `reason-scale-profile-2026-08-29.md` | **resolved** |
| Active PLAN lagged B1-B4 implementation | Roadmap now records B1-B3 and first B4 translator boundary as foundation; B5 performance is next | `PLAN.md` | **resolved** |
| Swarm onboarding still mixed old `:9999` coordination with new `swarm-node` authority | Added machine-readable legacy deprecation + replacement map, two-plane regression, and shortened `AGENTS.md` to one current onboarding model | `swarm-legacy-deprecation.wsm`, `swarm_deprecation.rs`, `documentation_contract.rs`; CI #1044/#1046 | **deprecation/migration contract fixed + confirmed; physical removal gated** |

## Current Advice Taker path

```text
external text / external model
        ↓
versioned (translation/1 ...) proposal
        ↓
translation-review                 ← Lisp-owned, pure
        ↓
accepted knowledge candidate?
   ├── no  → structured translation evidence
   └── yes → explicit advise / advise-all
                  ↓
          validated knowledge journal
                  ↓
           reason-in-observe
                  ↓
┌─────────┬─────────┬─────────┬─────────┬──────────┬─────────┐
│ proved  │ unknown │ partial │ blocked │ disputed │ invalid │
└─────────┴─────────┴─────────┴─────────┴──────────┴─────────┘
                  ↓
           narrate-outcome
```

The old `reason` / `reason-in` result shape still exists intentionally for
compatibility. No language-contract or evaluator exception rewrite was needed.

## Confirmed boundary improvements that are NOT complete-sandbox claims

### 1. Filesystem / TCP capability scoping

The core/host split is real: `crates/my-lisp` does not install OS capabilities;
`my-lisp-host` does. Per-session embedding policy now supports:

```text
filesystem read roots
filesystem write roots
tcp connect host/port ranges
tcp listen address/port ranges
process allowlist
```

Canonicalization and OS enforcement stay in `my-lisp-host`; semantic code does
not get to authorize itself. Symlink escape and `load`-as-read-bypass are
covered adversarially. The trusted native default remains unrestricted when no
scope is supplied, preserving compatibility.

This does **not** prove a complete sandbox. User-facing CLI syntax/defaults and
any stronger unauthenticated-oracle policy remain explicit operational design
choices, not hidden semantic changes.

### 2. Legacy coordination code in the semantic CLI

`swarm-node` is the intended coordination plane; `my-lisp :9999` remains the
semantic oracle. The old CLI still physically contains deprecated coordination
operations for compatibility.

Already confirmed:

1. operational onboarding moved to `swarm-node`;
2. deprecation is machine-readable in `knowledge/swarm-legacy-deprecation.wsm`;
3. replacement/two-plane behavior has regression coverage;
4. `AGENTS.md` no longer teaches legacy mailbox/task coordination as the
   current first-class path.

Physical deletion still requires **no-live-callers evidence**. After that,
remove broker/claims/presence/task coordination from `:9999` while keeping
`eval`/`parse`/`diagnose` semantic-oracle behavior unchanged.

### 3. Concrete TCP handles in core `Value`

`Value::TcpConnection` / `Value::TcpListener` still couple host-resource
representation to the core runtime type even though capability invocation is
host-gated. With only the current resource classes this is a structural risk,
not proof of semantic leakage.

**Guardrail:** do not add an open-ended sequence of device-specific `Value`
variants. A third/new family of opaque host resources should trigger an
explicit representation review (opaque handle/resource table or equivalent)
before extending `Value` mechanically.

### 4. Remaining evaluator head-dispatch surface

The first-class builtin migration is substantial but not mechanically complete.
Some deterministic forms remain evaluator-dispatched. They are not promoted to
primitive semantic identity merely by living in Rust.

Per `PLAN.md`, move such behavior only when it reduces duplicate semantic
authority, satisfies conformance, or directly helps the Advice Taker path.
"Shrink Rust" is not a line-count contest.

### 5. Arbitrary later-binding visibility in `meta-eval`

The current finite recursive-group representation proves self recursion and
contiguous mutual recursion without cyclic host environments. It does **not**
yet prove the stronger property that an earlier top-level closure sees every
arbitrary later binding across intervening forms. The existing `square` test
only proves definition followed by later invocation; it is not that stronger
forward-reference claim.

A future solution requires a deliberate lexical finite-data design, not an
accidental switch to dynamic scope or a mutable host frame.

This remains **unknown / explicit self-hosting proof gap**, not a defect to fix
by weakening lexical semantics.

### 6. Generated/historical documentation

Dated audits and generated references are lower semantic authority than current
contract + executable evidence. They should remain as historical evidence, but
must carry dates/statuses and must not override live behavior.

`docs/FUNCTIONS.md` identifies its baseline and incremental 2026-09-07 refresh.
The documentation regression prevents the actively changed reasoning,
narration and translation sections from silently drifting again. A full
repository-wide regeneration remains housekeeping, not semantic authority.

## CI evidence checkpoints

During this repair sequence, failures were treated as evidence rather than
papered over:

- outcome tests first caught an invalid use of `eq` on a pair; validation was
  changed to the atom-first pattern already used by `knowledge.my`;
- zero-warning clippy caught an unnecessary explicit Rust lifetime in the Guard
  validator;
- Guard tests caught a malformed negative fixture before the validator itself;
- swarm deprecation regression caught an extra closing parenthesis in the WSM
  marker;
- B4's first test run caught an invalid Rust assumption that parsed `Expr` had
  `Display`; the corpus check was kept parse-based instead of weakening it;
- the next B4 run showed all nine semantic boundary tests green and caught only
  a naive raw substring case count that counted the fixture comment; the count
  was made structural by line shape rather than changing the corpus claim.

Confirmed green checkpoints include:

- CI #1007 — tail-safe reason scan regression;
- CI #1019 — outcomes + narration + Guard + FASL hardening on one head;
- CI #1020 — seven-case end-to-end Advice Taker corpus;
- CI #1022 — malformed explicit-negation validation;
- CI #1030 — repository-wide repair ledger head;
- CI #1034 — refreshed default-stack scale harness + documentation drift gate;
- CI #1038 — per-session filesystem/TCP host capability enforcement;
- CI #1044 — machine-readable swarm deprecation after parser repair;
- CI #1046 — current two-plane agent onboarding + documentation regression;
- CI #1055 — Lisp-owned translation boundary, versioned corpus, translation
  inventory gate, workspace tests/build and zero-warning clippy all green;
- WASM browser #31 on the #1055 head — Chrome and Firefox green.

## Epistemic conclusion

The audit did **not** prove that `my-lisp` is complete, universally scalable,
fully sandboxed or fully self-hosted. It did establish stronger, narrower
facts:

```text
reason full-scan stack crash (tested boundary)      fixed
structured Advice Taker outcomes                    confirmed
failure/conflict presentation                       confirmed
7-mode end-to-end Advice Taker corpus               confirmed
external translator direct semantic authority       denied by tested Lisp boundary
Guard substring spoof                               fixed
committed core FASL freshness                       guarded by test
WASM semantic-change browser coverage               confirmed
scale harness historical-stack workaround           removed + confirmed
reasoning/narration/translation function drift      guarded
per-session FS/TCP embedding scopes                  confirmed on tested boundary
swarm legacy coordination deprecation               confirmed; deletion gated
arbitrary later binding                             explicit proof gap
opaque host-resource representation                 review trigger
```

The repository should now expand from this stronger center, not reopen already
closed failures without new contradictory evidence.
