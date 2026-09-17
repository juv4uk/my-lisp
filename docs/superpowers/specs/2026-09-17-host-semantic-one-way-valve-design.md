# Rust One-Way Retirement Valve Design

**Date:** 2026-09-17

**Parent:** #299

## Purpose

This document records the current owner directive for the active host-retirement phase.

The earlier semantic-only valve was too weak because it left `mechanism`, `boundary-data`, `observer`, ABI and machine work as possible routes for new Rust growth. The current rule is intentionally simpler:

```text
Lisp may grow.
Existing Rust may remain where still necessary.
Rust may only get smaller.
```

This text supersedes the earlier version of this document wherever it allowed new Rust mechanism or boundary additions.

## Outer invariant

For every repository change:

```text
if path ends with .rs:
    added lines must equal 0
    new .rs paths must equal 0
```

Therefore:

```text
Rust N+1 ⊆ Rust N
```

Operationally this is a source-diff valve, not merely a semantic-authority metric.

## What is allowed

Current Rust may be left unchanged when it is still required.

Rust may be reduced by:

- deleting stale assertions;
- deleting dead tests;
- deleting obsolete semantic producers/converters;
- deleting an entire `.rs` file when its remaining value has moved elsewhere;
- removing Rust-owned cardinality/shape authority after Lisp-owned evidence exists.

Lisp-owned code/data, contracts, witness corpora, documentation and non-Rust launch/policy infrastructure may grow as needed, subject to their own authority rules.

## What is forbidden during this phase

No Rust classification is an exemption. The following are all RED if they add Rust lines:

- semantic code;
- mechanism helpers;
- ABI/resource code;
- transport or external-boundary code;
- machine decoders/executors;
- Rust test observers;
- Rust comments added to an existing file;
- new Rust tests;
- new `.rs` files;
- replacement Rust introduced while deleting old Rust elsewhere.

A valuable Rust-growing branch is preserved, not discarded, but it does not merge as-is while this valve is active.

## Classification still matters

#304 classifies existing Rust as:

```text
semantic-producer
semantic-converter
compatibility-bridge
boundary-data
mechanism
observer
dead/stale
```

The classification answers one question: **what must be understood or preserved before deletion?**

It does not answer whether new Rust may be added. Under the outer valve, the answer to that second question is always no.

## Deletion safety

The valve must not erase the last executable copy of a useful semantic or scientific law.

For a live Rust site selected for retirement:

1. establish whether the site is reachable/current;
2. identify any unique useful law it carries;
3. if a unique law exists, preserve it first in Lisp-owned evidence;
4. use an already-existing mechanism or non-Rust launch path to observe the Lisp-owned named verdict;
5. delete the Rust copy without adding replacement Rust;
6. verify the relevant current semantic and mechanism lanes.

A dead/stale unreachable site may be deleted directly after sufficient reachability evidence.

## Boundary and mechanism Rust

Existing boundary/mechanism Rust may remain unchanged while needed. This design does not demand a flag-day evaluator rewrite.

However, `boundary-data`, `mechanism`, `observer`, `ABI`, `machine`, or `resource` labels do not permit growth during this phase. If new capability requires new Rust, that work is deferred or expressed through existing lower mechanism plus Lisp/data where possible.

## CI structure

#300 owns the executable outer valve.

It runs before semantic classification and must fail closed:

```text
new .rs file                 -> RED
any added line in *.rs       -> RED
Rust deletion-only diff      -> eligible for GREEN
no Rust touched              -> continue normal CI
```

This outer rule composes with existing inner guards:

```text
#115 host-test semantic-authority valve
#112 Lisp-owned expected meaning
#300 all-Rust subtraction valve
#304 deletion-safety inventory
```

The #300 implementation itself must not add Rust.

## Current canonical examples

`main@f5ff9240...`:

```text
crates/my-lisp/tests/unify.rs   +0 / -14
```

The obsolete host test was cut; no replacement Rust was written.

PR #303:

```text
Rust                               +0 / -20
Lisp witness corpora               grow
```

Host-side corpus cardinality authority is deleted while useful laws move into Lisp-owned corpora.

PR #308:

```text
exact_quantity_arithmetic.rs       deleted
Lisp exact-quantity witness         added
```

The Planck×Cs exact-energy law and the speed-of-light product/quotient inverse are preserved above the valve before the Rust semantic oracle disappears.

## Machine/backend work

Machine/backend branches may contain valuable Lisp encoders, evidence and research alongside Rust additions. Such branches belong to the preservation set.

During this phase:

```text
preserve branch/value
freeze Rust-growing merge
harvest Lisp-only value where safe
resume/decompose later under an explicit future policy
```

Do not discard valuable work merely because the current valve blocks its Rust-growing form.

## Agent ownership

- **Local agent:** broad reachability inventory, grep, deletion-only Rust cleanup, long local verification.
- **Web/integration agent:** preservation decisions, issue/contract synchronization, Lisp-owned witness migration, exact-head integration review.
- **Machine/backend agents:** preserve branches and continue non-Rust analysis where useful; do not merge new Rust while the valve is active.

## Acceptance

The active design is correctly implemented when:

- no PR can add a Rust line;
- no PR can add a new `.rs` path;
- deletion-only Rust changes remain possible;
- unique laws survive in Lisp before their Rust copies are cut;
- existing necessary Rust can remain unchanged;
- machine/backend Rust-growing work is preserved but frozen;
- #300 enforces the outer invariant mechanically;
- no lower-level classification can act as an exemption.

## Governing sentence

**Lisp may grow. Rust may remain where necessary. While the host is being sculpted, Rust only gets smaller.**
