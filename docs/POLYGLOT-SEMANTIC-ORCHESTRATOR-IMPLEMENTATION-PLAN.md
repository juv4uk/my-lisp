# Polyglot semantic orchestrator — implementation plan

**Status: PLAN ONLY.** No `POLYGLOT-002`-equivalent implementation
lands in this commit. Per direct owner instruction 2026-09-11: the
transport PoC (`crates/my-lisp-host/examples/foreign_python_probe.rs`,
commit `fc84888`) proved persistent-subprocess feasibility — that fact
is kept as one input to this plan, not treated as the seed the rest of
the architecture should grow from unreviewed. Three design decisions
already written into `docs/FOREIGN-RUNTIME-PROTOCOL-DESIGN-2026-09-11.md`
(`Value::Foreign`, `foreign-*` as Rust host capabilities, `(runtime,
handle)` as the `eq` identity rule) are explicitly **not ratified** —
Phase 0 below re-examines all three before anything is built on them.

## Target architecture (unchanged from the design doc, restated once)

```text
                         MY-LISP
                    semantic authority
                         👑
                          │
        ┌─────────────────┼─────────────────┐
        │                 │                 │
        ▼                 ▼                 ▼
 own implementation   foreign runtimes   host mechanisms
     backends
        │                 │                 │
 Rust/CML/WASM/FPGA   Python/Julia/JS    OS/FS/TCP/etc.
                          │
                    NumPy/SciPy/Astropy
```

My-lisp defines what things mean and orchestrates who computes them.
No implementation backend and no foreign runtime becomes a second
semantic authority by accident. Success is measured by **whether a new
backend or foreign-runtime adapter can be added without changing
semantic truth already established** — not by lines of Rust deleted or
added.

## Explicit architectural guards (apply to every phase below)

The following are regressions, not implementation details, and must
never land regardless of which phase is in progress:

- `Value::Python`, `Value::NumPy`, or any other runtime-specific
  `Value` variant.
- `match runtime { "python" => ..., ... }` inside the evaluator or any
  `crates/my-lisp` core code.
- A Python (or any foreign) module/function name appearing in Canon or
  `lib/surface/semantic-registry.wsm`.
- A foreign object's raw pointer/address becoming Lisp identity.
- Rust source acting as the *specification* for a foreign API (the
  foreign library's own documentation/behavior is the spec; Rust only
  transports).
- A second, separate semantic corpus per backend (one corpus, checked
  against N backends — `docs/BACKEND-NEUTRAL-SEMANTIC-ORACLE-2026-09-11.md`'s
  own model, generalized in Phase 5).

## Phase 0 — re-examine the three unratified design decisions

**Goal**: decide, with a real comparison (not a single default), what
`docs/FOREIGN-RUNTIME-PROTOCOL-DESIGN-2026-09-11.md` should have said
before any code depends on it.

**Architectural owner**: my-lisp (this repo) — these are semantic/API
surface decisions, not host-adapter details.

### Decision A: foreign-value representation

Compare, don't default:

| Model | Semantic neutrality | Portability (Rust/CML/WASM/FPGA) | Lifetime | Equality | Serialization | Capability isolation | New runtime = core change? | GC/RC interaction | NaN-box/layout interaction | First-class Lisp value? |
|---|---|---|---|---|---|---|---|---|---|---|
| **A. `Value::Foreign{runtime, handle}`** (design doc's original proposal) | Medium — bakes "foreign" into the core `Value` enum permanently | Every backend must know this variant exists | Rc-based, same as other Value variants | `eq` via `(runtime, handle)` | Needs a variant-specific rule | Registry-level, not type-level | No (good) | Same as Closure/TcpConnection | Needs a `layout.rs` NaN-box tag, like `Value::Builtin`'s `TAG_PRIMITIVE` did | Yes |
| **B. Generic opaque capability/resource value** (one existing `Value` shape, e.g. extending the opaque-handle pattern `TcpConnection` already uses, parameterized by a resource-kind tag rather than a new variant per resource) | High — "opaque resource" is already an existing, understood category (TCP handles), foreign values become one more instance, not a new concept | Same core shape already crosses Rust/host-adapter boundaries today | Same Rc-based pattern already proven | Identity via the same `Rc::ptr_eq` pattern already used | Same open question as A | Same as A | No | Same as existing opaque handles | No new NaN-box tag category needed if it reuses `TAG_PRIMITIVE`'s pattern | Yes |
| **C. Lisp-level foreign reference + minimal opaque host token** (the token itself is NOT a `Value` variant at all — it's data the *host adapter* tracks internally, referenced from Lisp only via a capability call, e.g. `(foreign-call foreign-runtime-handle "sqrt" 9)` where `foreign-runtime-handle` is itself an ordinary opaque value from Model A/B) | Highest — the language core has zero new concept; "foreign" is entirely an adapter-side bookkeeping detail | Best — a new backend needs to know nothing about foreign runtimes unless it specifically wants to support them | Adapter-managed table, not Rc at all — needs its own release discipline (this is real added complexity, not free) | Whatever the adapter defines — NOT observable via core `eq` at all unless re-exposed through B | N/A — nothing crosses into core `Value` | Cleanest — adapter can enforce whatever isolation it wants without touching core | No (best) | None — the core `Value`/GC system never sees these at all | None | No — this is the tradeoff: a Lisp program can't hold "the foreign thing" as an ordinary value, only as an adapter-side lookup key |

**Recommendation for Phase 0 to actually decide (not pre-decided
here)**: Model B is the strongest fit for the stated goal ("universal
opaque-resource mechanism that later suits Python, Julia, JS, JVM
equally") — it reuses an already-proven pattern (`TcpConnection`'s own
identity/lifetime rules) instead of inventing a new `Value` category,
and unlike Model C, foreign references remain ordinary first-class Lisp
values (needed for `(define np (foreign-import 'python "numpy"))` to
work naturally). Model A is what the PoC-era design doc assumed by
default; Phase 0's job is to confirm or reject B over A with a written
decision, not silently keep A because it was written down first.

### Decision B: where does `foreign-import`/`foreign-call`/`foreign-release` live?

The design doc's original framing (`foreign-import`/`foreign-call` as
direct Rust `my-lisp-host` capabilities, mirroring `process-run-raw`)
is too high a semantic layer for Rust, per the owner's correction.
Target split:

```text
public Lisp semantics (lib/*.my, Lisp-owned):
  foreign-runtime
  foreign-import
  foreign-member
  foreign-call
  foreign-release
  foreign-value?
        ↓
minimal raw host mechanism (my-lisp-host, Rust-owned):
  foreign-open-raw
  foreign-send-raw
  foreign-recv-raw
  foreign-close-raw
```

Rust's job shrinks to exactly what Lisp cannot do itself: spawn a
process, own a pipe/socket, hold an opaque OS-level resource handle.
Every higher-level concept (what "import" means, how a member is
resolved, how a call's arguments are encoded, how a response is
decoded into a Lisp value) is Lisp reading/writing the wire protocol
itself (Decision from Phase 2) over the four raw primitives above —
the same "narrow mechanism, Lisp-owned meaning" split already applied
to `read-file`/`process-run`/`tcp-read` (`crates/my-lisp-host/src/lib.rs`'s
own doc comments already describe exactly this pattern for existing
capabilities — this is not a new principle, it's applying an existing
one consistently to the new capability).

### Decision C: is `(runtime, handle)` `eq`-identity a contract fact?

Not automatically. Phase 0 must keep four things distinct, in writing,
before any of them becomes an observable `eq` rule:

1. **my-lisp's own semantic identity** — untouched by any of this.
2. **The Lisp-level foreign-reference value** (Model B/C above) — what
   a `.my` program actually holds and passes around.
3. **The adapter-local handle** — an implementation detail of whichever
   Rust adapter manages the raw connection, never guaranteed stable
   across process restarts.
4. **The foreign runtime's own object identity** (CPython's `id()`,
   say) — never my-lisp's concern at all.

Only after Phase 0 explicitly maps which of 2/3/4 (if any) `eq`
observes should that become a documented, tested contract fact —
avoiding the exact mistake already corrected once this session
(`Rc::ptr_eq` almost becoming the model for `Value::Builtin` identity).

**Phase 0 deliverable**: a short decision record (this document updated
in place, or a dedicated `docs/adr/ADR-0NN-FOREIGN-VALUE-MODEL.md`)
choosing one of A/B/C for Decision A, confirming or revising the
Decision B split, and stating the Decision C identity rule explicitly.
**Executable witness**: none required for Phase 0 itself (it's a
decision record) — but the decision must be falsifiable by Phase 1's
own acceptance criteria.
**Acceptance**: a named decision exists for A, B, and C, each with the
comparison table (or equivalent reasoning) that led to it, not just a
restated preference.
**Rollback**: none needed — this phase produces no code.
**Out of scope**: implementing any of the three — that's Phase 1+.

## Phase 1 — versioned wire protocol design (language-neutral)

**Goal**: design the actual bytes that cross the raw host boundary,
independent of Python specifically. The PoC's line protocol (`call
math sqrt 9`) is feasibility evidence only, not a production protocol
proposal.

**Architectural owner**: my-lisp (protocol semantics belong to the
language layer per Decision B), with input from whichever adapter(s)
exist at decision time.

**Compare, don't default**: S-expression-based (reusing this
ecosystem's own existing `.wsm`-family reader/writer machinery,
`sr/1`-style schema tags), framed JSON (widely supported by target
foreign runtimes, but a second serialization format this ecosystem
would now maintain), or a custom versioned binary wire format
(most efficient, most work, least reusable off-the-shelf tooling on
the foreign-runtime side). Recommendation to actually make in this
phase, not pre-made here: S-expression-based is the natural fit given
this ecosystem's own `sr/1`/`oracle-result/1`-style schema-tagged
protocol conventions already used for the compiler oracle export and
the swarm's own TCP protocol — reusing an established pattern instead
of introducing JSON as this project's first non-S-expression wire
format.

**Must account for** (from every case named in the owner's review, not
narrowed): protocol version tag, request ID (even before real
concurrency exists, so a future multi-outstanding-request extension
doesn't require a breaking protocol change), structured errors (not
bare strings — a foreign exception needs at least a kind/class and a
message, mirroring my-lisp's own `LanguageError` shape), exact rational
values (never silently downgraded to float crossing the boundary —
S1 applies at this boundary exactly as much as anywhere else in the
language), Unicode (both directions), lists/maps/vectors as compound
value shapes, large/binary data (at minimum a documented size class
where the versioned protocol explicitly declines to inline the data
and instead hands back a reference — Phase 6's concern in detail, but
the protocol's own extensibility must not preclude it), foreign-object
references (Decision A/B from Phase 0), malformed/untrusted response
handling (fail-closed — a foreign runtime that sends garbage must
produce a named error, never be silently accepted or crash the whole
session), and foreign-runtime crash/restart (the raw boundary must let
an adapter detect "the subprocess died" as a distinct, named condition
from "the subprocess returned an error response").

**Explicitly deferred, not designed as unnecessary**: real async/
concurrent request handling. The protocol must not *preclude* adding
it later (hence the request-ID field even now), but nothing in Phase
1-4 implements overlapping in-flight requests.

**Files touched (approximate)**: a new schema document (e.g.
`docs/foreign-runtime-wire-protocol-v1.md`, mirroring
`docs/cml-semantic-export-v1-design.md`'s own "format fixed before
code" structure), no `crates/` changes yet.

**Depends on**: Phase 0's Decision A/C (the protocol must be able to
carry whatever the foreign-reference model needs to cross the wire).

**Acceptance**: the protocol document lets someone answer, on paper,
"what bytes would cross the wire for `(foreign-call math \"sqrt\" 9)`
and its response" including version/request-ID/error-shape, without
needing to read any Rust source.

**Executable witness**: none required yet — Phase 2 proves it for
real.

## Phase 2 — raw host mechanism (Rust, minimal)

**Goal**: implement exactly `foreign-open-raw`/`foreign-send-raw`/
`foreign-recv-raw`/`foreign-close-raw` as `my-lisp-host` capabilities,
allowlist-gated the same way `process-run-raw`/`tcp-connect` already
are. No import/call/member semantics here — those are Phase 3's Lisp
code.

**Architectural owner**: `my-lisp-host` (Rust) for the raw mechanism
only.

**What belongs to Lisp / what's allowed in Rust**: Rust owns spawning
the persistent subprocess, holding its stdin/stdout as an opaque
resource, and moving bytes across the wire-protocol framing boundary
(Phase 1) — nothing about what those bytes *mean*. Reusing
`process_raw.rs`'s existing allowlist check
(`environment.is_process_allowed`) rather than inventing a second
security mechanism.

**Files touched (approximate)**: new `crates/my-lisp-host/src/foreign_raw.rs`
(mirroring `process_raw.rs`'s own structure), registration in
`crates/my-lisp-host/src/lib.rs`.

**Depends on**: Phase 0 (Decision B confirms this is the right layer),
Phase 1 (the framing these primitives move bytes according to).

**Executable witness**: a Rust integration test proving the four raw
primitives work end to end against the *same* Python bridge script the
PoC already proved feasible — reusing, not discarding,
`foreign_python_bridge.py`'s protocol shape (upgraded to Phase 1's real
framing, not the PoC's bare line format).

**Acceptance**: `foreign-open-raw`/`foreign-send-raw`/`foreign-recv-raw`/
`foreign-close-raw` are the *only* new host capabilities this phase
adds — no `foreign-import`/`foreign-call` capability, confirming
Decision B held.

**Rollback/fail-closed**: an unopened or already-closed raw handle used
in any of the four operations is a named error (`InvalidForm`-shaped,
matching existing capability error conventions), never a panic or
silent no-op.

**Out of scope**: any Lisp-visible `foreign-*` public API — that's
Phase 3.

## Phase 3 — Lisp-owned foreign API

**Goal**: `foreign-runtime`, `foreign-import`, `foreign-member`,
`foreign-call`, `foreign-release`, `foreign-value?` as ordinary `.my`
library functions built entirely on Phase 2's four raw primitives —
the evaluator gains zero new special-form knowledge.

**Architectural owner**: my-lisp (this is language-owned semantic
surface, per Decision B).

**Files touched (approximate)**: new `lib/foreign.my` (mirroring
`lib/process.my`/`lib/tcp.my`'s own "language interprets raw host
bytes" pattern), a loader function in `crates/my-lisp/src/lib.rs`
analogous to `load_process_library`.

**Depends on**: Phase 2.

**Executable witness**: `(foreign-call math "sqrt" 9)` returning `3`
(or the correct exact-rational-aware equivalent per Phase 1's protocol)
through the full Lisp-owned call path, with the evaluator provably
unaware of `python`, `math`, or `sqrt` (a test asserting no
Rust-side string match on any of those three names exists in the
`foreign.my` loading path).

**Acceptance**: everything in "What belongs to Lisp" from the design
doc's original list is now implemented in `.my`, not Rust.

## Phase 4 — value conversion + reference lifetime

**Goal**: the small closed set of value shapes (exact/inexact number,
string, symbol where meaningful, `t`/`()`, list, foreign-reference)
converts correctly across the boundary in both directions, and foreign
references have a real, tested release discipline.

**Depends on**: Phase 3, Phase 0's Decision A (which representation
model foreign references use).

**Executable witness**: round-trip tests for every value shape in the
closed set, plus a test proving a released foreign reference cannot be
used again (must produce a named error, not silently succeed against a
dangling adapter-side entry).

**Out of scope**: any binary/large-data special-casing — that's
Phase 6's NumericBuffer bridge, deliberately kept separate per the
owner's own instruction not to mix it with the base call protocol.

## Phase 5 — Python acceptance witness

**Goal**: the concrete acceptance program from the design doc, for
real, end to end:

```lisp
(define math (foreign-import 'python "math"))
(foreign-call math "sqrt" 9)
```

then, as the next level (not simultaneously):

```lisp
(define np (foreign-import 'python "numpy"))
```

proving a real NumPy object round-trips as a foreign reference (not
yet doing anything with its data — that's Phase 6).

**Explicitly not started in this phase**: SciPy/Astropy/PyTorch. Per
the owner's own instruction, the base Python adapter must be proven on
one shared protocol before any second library is attempted.

**Acceptance**: both acceptance snippets above execute successfully
against the real, running implementation from Phases 2-4, as an
automated test, not a manually-run demo.

## Phase 6 — NumericBuffer ↔ NumPy bridge (separate research track)

**Goal**: investigate whether `crates/my-lisp/src/value.rs`'s existing
`NumericBuffer` (`i32-buffer`/`f32-buffer`) can bridge to Python's
buffer protocol / NumPy's `ndarray` without a full copy through Lisp
lists for large arrays.

**Explicitly a research phase, not an implementation commitment**:
compare plain copy (simplest, works with the Phase 1-4 protocol as-is,
correct but not zero-copy), shared memory / mmap (real zero-copy, but
the subprocess transport model from Phase 2 makes true shared memory
between two separate OS processes nontrivial — needs its own design,
not assumed to just work), and in-process Python embedding (PyO3,
a genuinely different backend adapter model than the subprocess
approach Phases 2-5 build, not a variant of it — would need its own
Phase-0-style re-examination of the raw-mechanism boundary, since an
embedded interpreter changes what "the host mechanism Lisp cannot do
itself" even means).

**Deliverable**: a benchmark/correctness prototype comparing at least
plain-copy vs. one zero-copy candidate, informing a real decision — not
built as production code in this phase.

**Depends on**: Phase 5 (a working Python adapter to attach the buffer
question to) — but is not on the critical path to Phase 5 landing.

## Phase 7 — backend-neutral oracle interface (parallel track)

**Goal**: generalize `docs/BACKEND-NEUTRAL-SEMANTIC-ORACLE-2026-09-11.md`'s
current native-Rust-vs-meta-eval.my pair into a real N-backend
interface, so a third/fourth backend (CML, eventually WASM/FPGA) can
attach to the same semantic corpus without rewriting the comparison
harness per backend.

**Explicitly named limitation from the prior work, carried forward
honestly**: `lib/meta-eval.my` is not an independent backend proof —
it's a Lisp program interpreted by the same native Rust evaluator it's
compared against. A real second data point requires a backend that
does not share that evaluator (CML, at minimum).

**Deliverable**: a standard result/evidence record shape (mirroring
`oracle-result/1`'s own existing schema-tag convention) any backend
adapter implements, so `crates/my-lisp/tests/compiler_corpus_dual_backend.rs`'s
own pattern generalizes to N backends without N different comparison
harnesses.

**Depends on**: nothing from Phases 0-6 — this can proceed in parallel,
using #67's already-frozen compiler-corpus as the shared input.

**Next concrete backend after this interface exists**: a CML oracle
adapter (cml's own existing conformance work, already independently
checking against real my-lisp fixtures per this session's earlier
coordination, is the natural first real second-implementation-not-
sharing-Rust proof).

## Phase 8 — release gate

**Goal**: the point at which "Python-first foreign runtime support" is
a real, documented, tested feature, not a research branch.

**Acceptance**: Phases 1-5 executable witnesses all pass together in
one CI-runnable suite; the architectural guards list above has a
corresponding negative test for each guard (mirroring
`crates/my-lisp/tests/error_kind_vocabulary_is_closed.rs`'s own
exhaustive-match-as-gate pattern) proving, e.g., that adding a
`Value::Python`-shaped variant would fail some check, not just that
nobody happened to add one yet.

**What can be physically deleted after this phase, if anything**:
`crates/my-lisp-host/examples/foreign_python_bridge.py`/
`foreign_python_probe.rs` — the PoC's own job (prove transport
feasibility) is fully subsumed by Phase 2's real, tested raw mechanism
by this point. Not deleted before then; it remains the only executable
evidence for feasibility until Phase 2 lands.

## Dependency graph (informal)

```text
Phase 0 (decisions A/B/C)
  ├── Phase 1 (wire protocol) ── depends on 0.A, 0.C
  │     └── Phase 2 (raw mechanism, Rust) ── depends on 0.B, 1
  │           └── Phase 3 (Lisp-owned API) ── depends on 2
  │                 └── Phase 4 (conversion + lifetime) ── depends on 3, 0.A
  │                       └── Phase 5 (Python acceptance) ── depends on 4
  │                             ├── Phase 6 (NumPy bridge research) ── depends on 5, not blocking
  │                             └── Phase 8 (release gate) ── depends on 5

Phase 7 (backend-neutral oracle interface) ── independent, parallel to all of the above
```

## What is intentionally not decided by this document

This plan does not choose the wire format, does not choose the
foreign-value model, and does not choose the `eq` identity rule —
those are Phase 0's job, to be done as real comparative work, not
retrofitted justification for what the PoC happened to do first.
