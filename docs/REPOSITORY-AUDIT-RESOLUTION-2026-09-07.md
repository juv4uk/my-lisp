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
| `reason` full rule scan used non-tail `append(... recursive scan ...)`, overflowing ordinary stack in the measured worst case | `prove-goal` now delegates to a tail-recursive scan, accumulates with `cons`, reverses once | `reason_stack.rs`: 256-rule default-stack full scan + result-order regression; historical profile updated | **fixed + confirmed** |
| `reason`/`reason-in` collapsed multiple epistemic meanings into ordinary `()` | One canonical opt-in data algebra in `result-status.my`: `proved / unknown / partial / blocked / disputed / invalid`; compatibility APIs unchanged | `result_status.rs`, `reason_outcome_invalid.rs`, `reason_in_outcome_invalid.rs` | **fixed + confirmed on tested outcome boundary** |
| Proposed B1 vocabulary duplicated the older `unknown/partial/blocked/disputed` convention | Existing convention was extended instead of creating a second algebra; ADR updated | `docs/adr/unknown-result-semantics.md` | **resolved** |
| A successful query may have several answers; a new wrapper could accidentally keep only the first | `proved` stores the complete `reason` result list; dispute stores evidence for both sides | multiple-alternative test in `result_status.rs` | **fixed + confirmed** |
| Malformed goals could be mislabeled `unknown` | Goal observation now requires a proper list + symbolic predicate; `(not goal)` has exact recursive shape; module/goal validation precedes module lookup | malformed `not`, non-symbol head, invalid module and missing-module precedence tests | **fixed + confirmed on tested shapes** |
| `narrate` explained proofs but collapsed failure modes | `narrate-outcome` preserves explicit status in presentation for all six outcome classes | `narrate_outcomes.rs` | **fixed + confirmed** |
| Malformed tagged presentation data could crash/access missing fields | `narrate-outcome` validates proper list, tag type and exact tag arity before field access | truncated/unknown/non-symbol-tag adversarial tests | **fixed; final head CI required** |
| Advice Taker lacked one adversarial end-to-end corpus over the new outcome path | Added 7-case corpus: direct, multi-step, recursive, unknown, conflict rejection, malformed advice, knowledge-package round-trip | `advice_corpus.rs`; CI #1020 | **fixed + confirmed** |
| Committed `core.my.fasl` could drift from current `core.my` without a repository-wide invariant | Added exact embedded-source-hash regression | `core_fasl_freshness.rs` | **fixed + confirmed** |
| Browser/WASM suite did not run when semantic dependencies changed | Workflow path filters now include core runtime/literate/LSP/core source/FASL/Cargo graph | WASM browser runs after semantic changes, Chrome + Firefox | **fixed + confirmed** |
| `wsm-guard-core` accepted rendered substring `"(decision allow)"` rather than a real protocol value | Rust adapter validates exact `guard-finding`, `guard/1`, ordered fields, decision enum and evidence-status enum before rendering | spoofed nested decision + wrong schema tests | **fixed + confirmed** |
| Old reason scale report still read like the current broken state after repair | Report split into historical measurement vs confirmed 2026-09-07 follow-up | `reason-scale-profile-2026-08-29.md` | **resolved** |
| Active PLAN still said B1 was NEXT after B1-B3 landed | Roadmap advanced: B1-B3 moved into confirmed foundation; B4 becomes next semantic front | `PLAN.md` | **resolved** |

## Current Advice Taker path

```text
candidate data
    ↓
advise / advise-all
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

## Migration gates deliberately NOT disguised as fixes

### 1. Filesystem / TCP capability scoping

The core/host split is real: `crates/my-lisp` does not install OS capabilities;
`my-lisp-host` does. But once installed, filesystem and TCP permissions are
still broad compared with `process`'s per-session policy.

`docs/host-capability-scoping-adr-2026-08-27.md` remains **PROPOSED** for FS/TCP.
The repository audit does not silently ratify its open choices. Implementation
must follow an explicit compatibility/security decision, especially for:

- separate read vs write roots;
- symlink/canonicalization policy;
- connect vs listen policy;
- trusted native profile vs partially-trusted embedding defaults.

Until then: trusted native Lisp-machine use is the intended broad-capability
profile; partially-trusted autonomous execution is a **migration gate**, not a
property to claim complete.

### 2. Legacy coordination code in the semantic CLI

`swarm-node` is the intended coordination plane; `my-lisp :9999` remains the
semantic oracle. The old CLI `swarm.rs` still contains legacy coordination
operations. Deleting them immediately could break live callers.

Removal requires evidence in this order:

1. agents use `swarm-node` for coordination;
2. legacy operations are explicitly deprecated;
3. migration/replacement path is tested;
4. coordination operations are removed from `:9999` while semantic oracle
   behavior stays unchanged.

This is **partial migration**, not an unresolved semantic-core defect.

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

### 5. Generated/historical documentation

Dated audits and generated references are lower semantic authority than current
contract + executable evidence. They should remain as historical evidence, but
must carry dates/statuses and must not override live behavior.

`docs/FUNCTIONS.md` already identifies its library inventory base date rather
than claiming timeless authority. A full regeneration is housekeeping; it is
not a semantic prerequisite for the repairs above.

## CI evidence checkpoints

During this repair sequence, failures were treated as evidence rather than
papered over:

- outcome tests first caught an invalid use of `eq` on a pair; validation was
  changed to the atom-first pattern already used by `knowledge.my`;
- zero-warning clippy caught an unnecessary explicit Rust lifetime in the new
  Guard validator;
- Guard tests caught a malformed negative fixture before the validator itself;
  the fixture was corrected without weakening validation.

Confirmed green checkpoints include:

- CI #1007 — tail-safe reason scan regression;
- CI #1019 — outcomes + narration + Guard + FASL hardening on one head;
- CI #1020 — seven-case end-to-end Advice Taker corpus;
- CI #1022 — malformed explicit-negation validation;
- WASM browser runs after semantic-dependency trigger expansion — Chrome and
  Firefox green.

The newest head after this ledger must still pass workspace tests/build,
zero-warning clippy and the browser workflow before being called globally
confirmed.

## Epistemic conclusion

The audit did **not** prove that `my-lisp` is complete, universally scalable or
fully self-hosted. It did establish stronger, narrower facts:

```text
reason full-scan stack crash (tested boundary)      fixed
structured Advice Taker outcomes                    confirmed
failure/conflict presentation                       confirmed
7-mode end-to-end Advice Taker corpus               confirmed
Guard substring spoof                               fixed
committed core FASL freshness                       guarded by test
WASM semantic-change browser coverage               confirmed
fine-grained host security                          migration gate
legacy coordination removal                         migration gate
opaque host-resource representation                 review trigger
```

The repository should now expand from this stronger center, not reopen already
closed failures without new contradictory evidence.
