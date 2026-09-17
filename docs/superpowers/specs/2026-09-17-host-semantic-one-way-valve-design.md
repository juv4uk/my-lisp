# Host Semantic One-Way Valve Design

**Date:** 2026-09-17

**Parent:** #299

## Purpose

`my-lisp` is the semantic authority. Rust remains a strong and comfortable substrate for mechanism, transport, resources, ABI, safety and observation, but it may not gain new authority over what Lisp programs mean.

The core invariant is monotonic:

```text
Rust semantic authority at commit N+1 <= Rust semantic authority at commit N
```

This is intentionally analogous to the existing one-way retirement of host-authored semantic tests, but it applies to production/runtime/host implementation code.

## Constitutional rule

```text
Rust may report a mechanism fact.
Lisp decides what that fact means in the language.
```

Increasing host capability is allowed. Increasing host-owned language meaning is not.

## Authority classes

Every Rust site capable of affecting Lisp-visible results is classified as one of:

- `semantic-producer` — manufactures a language-domain result/answer/policy;
- `semantic-converter` — converts host/internal values into a language semantic interpretation;
- `compatibility-bridge` — explicitly named temporary legacy behavior;
- `boundary-data` — preserves externally-defined protocol data without interpreting it as Lisp meaning;
- `mechanism` — memory, parser/bootstrap mechanics, OS/ABI/resource/bytes operations;
- `observer` — launches or observes Lisp-owned contracts/witnesses;
- `dead/stale` — unreachable or superseded semantic mechanism awaiting deletion.

The inventory is descriptive evidence. It is not a second semantic registry and may not define Lisp meaning.

## Monotonic movement

Allowed:

```text
semantic-producer
   -> semantic-converter removed / Lisp-owned interpretation
   -> compatibility-bridge or boundary-data
   -> mechanism
   -> deleted
```

Forbidden:

```text
mechanism/boundary/deleted
   -> new semantic-producer
```

Also forbidden:

- replacing one Rust semantic token with another Rust semantic token;
- adding a new unnamed `Value -> bool`, `bool -> Lisp`, `t/()`, truthy, answer-shape or domain-classification bridge;
- adding host-side semantic tables/counts that constrain Lisp-owned contracts or witness corpora;
- adding a new Rust-only public builtin whose language meaning does not exist in Lisp-owned authority.

## Boundary data

External JSON/API/FFI booleans, integers and status codes may remain represented faithfully in Rust when the external protocol itself defines them.

That does not grant them Lisp truth semantics.

Example:

```text
external bool/status
      -> Rust boundary data
      -> explicit mechanism fact
      -> Lisp-owned wrapper/contract
      -> public Lisp result
```

The host reports what happened; Lisp decides what that fact means.

## Errors and mechanism failures

Mechanism failures remain explicit mechanism facts/failures. They must not silently collapse into semantic absence, `()`, false, `UnknownSymbol`, or a domain answer.

#250 is the first concrete honesty case: `Present`, `Absent`, and `Unreadable` must remain distinguishable at the mechanism boundary. Any language-domain interpretation belongs above that boundary.

## Deletion safety

The valve encourages deletion but must not destroy the last executable copy of a useful semantic law.

For every live Rust semantic site being removed:

1. determine whether it carries a unique law;
2. if yes, preserve that law in a Lisp-owned contract/witness first;
3. keep useful host/mechanism diagnostics;
4. delete or narrow the Rust semantic site;
5. prove the Lisp-owned witness remains green.

A dead/stale unreachable site may be deleted after reachability evidence without inventing a replacement law.

## CI structure

Two independent one-way valves compose:

```text
#115 test-authority valve
  host tests may become less semantic, never more semantic

#300 runtime-authority valve
  Rust runtime/host implementation may become less semantic, never more semantic
```

They should reuse classification data where practical, but remain separately testable.

The #300 guard must be fail-closed and evidence-based rather than a blanket grep. Required behavior:

- deletion/reduction of an inventoried semantic site -> GREEN;
- new/expanded semantic producer or converter -> RED;
- unclassified new Rust path capable of shaping Lisp-visible values -> RED;
- genuine boundary-data addition -> GREEN;
- mechanism/ABI/bytes/resource change -> GREEN;
- error names the exact site and violated authority class.

The policy verdict should remain Lisp-owned, following #115's precedent.

## First migration queue

1. #304 — inventory/freeze current Rust semantic authority as a ceiling;
2. #300 — implement the CI monotonicity valve;
3. #301 — migrate `tcp-close` away from Rust-owned `Value::Bool(true)` public meaning;
4. #305 — retire dead `atom` path and unnamed truth/semantic converters in `closures.rs` and `value.rs`;
5. continue small preservation-first retirement slices from #304 findings.

## Agent ownership

To avoid write races:

- local agent: exhaustive code inventory, reachability checks, broad grep, deep/full-package verification, mechanical deletion-only retirements after authority is settled;
- web/integration agent: contract interpretation, issue dependency updates, CI/authority review, preservation decisions, exact-head merge readiness;
- machine/backend agents: may continue expanding mechanism capability, but must not introduce new Lisp meaning in host/backend code.

## Acceptance

The architecture is established when all are true:

- new Rust semantic authority cannot be added silently;
- current semantic authority is machine-readably inventoried as a ceiling;
- semantic authority can only shrink or move to explicit compatibility/boundary/mechanism roles;
- unique semantic laws survive in Lisp-owned authority before Rust copies disappear;
- mechanism, ABI, transport, parser/bootstrap and resource work remain comfortable and low-friction;
- external booleans/data remain representable without becoming Lisp truth;
- #115 and #300 both pass without duplicating Lisp semantics in host policy code.

## Governing sentence

**Rust is the comfortable throne cushion: strong mechanism underneath, zero right to crown itself semantic authority. Lisp is the king.**
