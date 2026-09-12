# Foreign Runtime Protocol — design (fixed before code)

**ARCHIVED — Superseded-by: [`docs/POLYGLOT-SEMANTIC-ORCHESTRATOR-IMPLEMENTATION-PLAN.md`](../../POLYGLOT-SEMANTIC-ORCHESTRATOR-IMPLEMENTATION-PLAN.md).**
This document's three key decisions (`Value::Foreign`, `foreign-import`/
`foreign-call` as Rust host capabilities, `(runtime,handle)` as `eq`
identity) were explicitly un-ratified by the owner and re-examined from
scratch in the plan above, which found a zero-core-change architecture
via reuse of existing TCP/`write-to-string` mechanism. Kept here as the
historical record of the first design pass and the Python transport
PoC's own rationale — not as a current specification.

Owner directive 2026-09-11: my-lisp should become not just a
semantic authority over its own implementation backends (Rust/CML/
WASM/FPGA), but a polyglot semantic orchestrator able to drive foreign
runtimes and their libraries — Python first (NumPy/SciPy/Astropy/
PyTorch/etc.), without becoming a special case wired into the
evaluator. Per this repo's own established discipline (the CML
semantic export precedent, `docs/cml-semantic-export-v1-design.md`):
design fixed before code. This document is that fixed-before-code
step; only a minimal, isolated transport proof-of-concept accompanies
it (see "What is actually implemented" below) — the full feature is
explicitly not built in one pass.

## The three-layer separation this design commits to

```text
                  ┌─────────────────────┐
                  │      MY-LISP        │
                  │ semantic authority  │
                  └──────────┬──────────┘
                             │
          ┌──────────────────┼──────────────────┐
          │                  │                  │
          ▼                  ▼                  ▼
   implementation        foreign runtimes   host capabilities
      backends
          │                  │                  │
     Rust/CML/FPGA      Python/Julia/JS    FS/TCP/clock
```

These are three different concerns, deliberately kept apart:

- **Implementation backends** (Rust today, CML/WASM/FPGA later) execute
  my-lisp's *own* semantics — the same numeric semantic identity means
  the same thing regardless of which one runs it
  (`docs/BACKEND-NEUTRAL-SEMANTIC-ORACLE-2026-09-11.md`).
- **Foreign runtimes** (Python first) are NOT asked to implement
  my-lisp semantics at all. My-lisp calls into them and they do their
  own thing (NumPy's `sin` means whatever NumPy says it means) — my-lisp
  owns the *protocol of interaction*, never the foreign library's own
  semantics.
- **Host capabilities** (already existing: filesystem, process, TCP,
  clock) are the narrow, already-established mechanism category this
  new protocol is architecturally a sibling of, not a replacement for.

## What my-lisp owns vs. what it explicitly does not own

**Owns** (this is the actual semantic surface of the feature):
`foreign-import`, `foreign-call` (attribute/member resolution + call +
positional arguments), result conversion for a small closed set of
value shapes, an opaque foreign-value handle, explicit
release/lifetime, error translation into named `LanguageError`s.

**Does not own**: any individual foreign library's own functions.
`numpy.linalg.eig` is not, and must never become, part of Canon or the
semantic registry. This mirrors exactly the reasoning already settled
for `Value::Builtin`
(`docs/BUILTIN-IDENTITY-MIGRATION-MAP-2026-09-11.md`) and opaque
capability handles
(`docs/cyberpunk-opaque-capability-semantics.md`): a foreign object's
runtime pointer/handle is never semantic identity, the same discipline
already applied to Rust `Rc` pointers and RED4ext game handles.

## Value representation: one boundary, not a zoo

Explicitly rejected: `Value::Python`, `Value::Julia`, `Value::JS` as
separate architectural variants — this would make `Value` grow one new
case per foreign runtime forever, and (per the same reasoning as the
reverted `Value::Builtin` `Rc::ptr_eq` fix) would make each foreign
runtime's own object representation part of language identity.

Instead, one `Value::Foreign` shape, carrying:

```text
ForeignValue {
    runtime: <opaque runtime identifier, e.g. "python">
    handle:  <opaque numeric/token id, host-adapter-local>
}
```

`eq`/`equal?` on two `ForeignValue`s compares `(runtime, handle)`
identity — the same "resource handle is itself" rule already used for
`TcpConnection`/`TcpListener` (`crates/my-lisp/src/value.rs`), not the
foreign runtime's own object address. `runtime` is a string tag chosen
by the adapter that registered the capability, not a closed enum in
`crates/my-lisp` itself — adding Julia later needs no core-crate change,
only a new adapter registering capabilities under its own runtime tag.

## The protocol my-lisp actually needs (Python vertical slice)

Minimum operations, matching the acceptance criterion
`(foreign-call math "sqrt" 9)` without the evaluator knowing what
`sqrt` means:

1. Runtime startup/connect (spawn or attach to a running interpreter).
2. Import a module → returns a `ForeignValue`.
3. Resolve a member/attribute on a `ForeignValue`.
4. Call, with positional arguments — the arguments and the return value
   both cross a **value-shape boundary**, not a type-shape boundary:
   exact/inexact number, string, symbol (where meaningful), `t`/`()`,
   list, and `ForeignValue` (for anything my-lisp does not need to
   understand, just hold and pass back).
5. Explicit handle release (foreign objects are not garbage-collected
   by my-lisp's own Rc discipline — they live in a different runtime's
   memory).
6. Error translation: a foreign exception becomes a named
   `LanguageError` (which `ErrorKind` this maps to — likely a new kind,
   or `InvalidForm` reused — is an open question for the first real
   implementation attempt, not resolved here).

## Transport: why a persistent subprocess, not `process-run-raw`

`crates/my-lisp-host/src/process_raw.rs`'s existing `process-run-raw`
is a **one-shot** capability: spawn, wait for exit, collect
stdout/stderr, done. A foreign runtime needs a **persistent**,
bidirectional channel (import once, call many times, release handles
individually) — a structurally different capability, not a variation
of the existing one, though it should follow the same discipline
(allowlist-gated, narrow Rust mechanism, no policy smuggled into the
transport layer itself).

Confirmed directly in this environment: `python` (not `python3`) is
available, version 3.12.10 — a subprocess-based bridge is feasible
here without embedding CPython via FFI (no PyO3 dependency needed for
a first slice). The Python side of the bridge is a small, fixed script
(this repo's own mechanism, analogous to `asm/nucleus.s` for
wsm-my-lisp — a minimal, auditable adapter, not "the implementation").

## Zero-copy numeric bridge (flagged, not designed here)

`crates/my-lisp/src/value.rs`'s existing `NumericBuffer` (`i32-buffer`/
`f32-buffer`, already used for typed numeric data since contract 2.2)
is the natural candidate to bridge against Python's buffer protocol /
NumPy's `ndarray`, avoiding a copy through Lisp lists for large arrays.
This is real, valuable, and explicitly **not designed in this
document** — it requires understanding exactly how a subprocess-based
transport (not an in-process embed) can share memory at all (a
subprocess boundary makes true zero-copy hard; shared memory or
memory-mapped files would be the realistic mechanism, a genuinely
separate research question from the request/response protocol above).
Flagged for a dedicated follow-up once the basic call protocol is
proven, not before.

## What is actually implemented alongside this document

Nothing wired into the evaluator or `my-lisp-host`'s capability
registry — that would be exactly the "build the whole feature in one
pass" this repo's own `docs/agent-doctrine.md` rule 7 (minimize change
surface) and the vertical-slice discipline established this session
both warn against for a feature this large.

What accompanies this document is a single, isolated, standalone
transport proof-of-concept —
`crates/my-lisp-host/examples/foreign_python_probe.rs` — proving only
the riskiest unknown first: can a persistent Python subprocess be
spawned, sent one structured request over stdin, and produce a parsed
response over stdout, without CPython embedding. It does not touch
`Value`, the evaluator, or the capability registry. See that file's own
header for what it proves and what it deliberately does not.

## Next steps, not started here

1. Prove the transport PoC (this document's companion example).
2. Design `ErrorKind` mapping for foreign exceptions (real design
   question, not resolved here).
3. Add `Value::Foreign` to `crates/my-lisp/src/value.rs` with identity
   `eq` (mirroring `TcpConnection`), following the same audit discipline
   `docs/BUILTIN-IDENTITY-MIGRATION-MAP-2026-09-11.md` used — check
   every existing exhaustive `match` over `Value` before adding the
   variant, since several already exist (`PartialEq`, `presentation.rs`,
   `layout.rs`'s NaN-boxing, `language_items.rs`).
4. Register `foreign-import`/`foreign-call`/`foreign-release` as new
   `my-lisp-host` capabilities, allowlist-gated the same way
   `process-run-raw`/`tcp-connect` already are.
5. Only after 1-4 are proven: the NumericBuffer/NumPy zero-copy bridge.

Not attempted in this pass, and not recommended to attempt
speculatively: Julia/JS adapters. Per this repo's own "narrow scope,
one real need at a time" discipline (mirrored in every generator this
session touched — wsm-my-lisp's and cml's own `build.rs` scripts each
cover only the IDs their code actually special-cases, not the whole
registry) — a second foreign-runtime adapter should wait for a real,
concrete need, not be built ahead of one.
