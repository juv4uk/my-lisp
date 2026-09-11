# Polyglot semantic orchestrator — implementation plan

**Status: PLAN, revised 2026-09-11 (reuse-first audit).** This
supersedes the plan's original Phase 0-8 structure, which treated
`Value::Foreign`, a generic opaque resource, and four new
`foreign-*-raw` host primitives as alternatives to choose between. A
direct audit of this repository's own existing code found that the
choice was already made, years of `docs/agent-doctrine.md`-rule-7
discipline ago: **the substrate this feature needs already exists and
needs no core change at all.**

The transport PoC (`crates/my-lisp-host/examples/foreign_python_probe.rs`,
commit `fc84888`) is kept as one input/evidence for this plan — it
proved a persistent Python subprocess can hold a session across
multiple requests. It is not the seed the architecture below grows
from; the architecture below grows from `lib/knowledge.my`,
`lib/tcp.my`, and `lib/process.my`, which already prove the pattern
end to end for other purposes.

## What the audit found already built

- **Core is already capability-free.** `crates/my-lisp` itself has no
  filesystem/process/TCP capability; `my-lisp-host` registers narrow
  mechanisms behind a plain-function-pointer registry (no stateful Rust
  closures allowed). This is exactly the shape a foreign-runtime bridge
  needs, already in place.
- **`lib/process.my` already demonstrates `raw mechanism → Lisp
  semantics`**: `process-run-raw` (host, bytes only) →
  `process-run-text`/`process-run` (Lisp, UTF-8 + result policy).
- **`lib/tcp.my` already demonstrates the same split for a persistent,
  bidirectional channel**: `tcp-connect`/`tcp-read-raw`/`tcp-write-raw`/
  `tcp-close` (host, bytes/handle only) → `tcp-read`/`tcp-write` (Lisp,
  UTF-8 encoding/decoding). `TcpConnection` is already an opaque,
  first-class host resource — exactly the shape a foreign-runtime
  connection needs, already in place.
- **Lisp-owned framing over TCP already exists and is battle-tested**:
  `lib/knowledge.my`'s `tcp-read-frame` (newline-delimited, accumulates
  partial reads) and `exchange-knowledge-package`/
  `accept-knowledge-exchange` (write one framed canonical S-expression,
  read one framed canonical S-expression back). This is a working
  request/response protocol over TCP, owned entirely by Lisp, with zero
  Rust awareness of what is being exchanged.
- **`write-to-string`/`read` already form a ratified,
  implementation-independent canonical wire format**, with the standing
  contract `read(write-to-string(value)) == value` for `()`, `t`,
  symbols, strings, lists, dotted pairs, exact integers, exact
  rationals, and finite inexact numbers (`crates/my-lisp/src/eval/special_forms/io.rs`,
  `language_items.rs`). No second serializer needs inventing.
- **`docs/host-portability-contract.md`-style division already states
  the rule this feature is an instance of**: host owns
  mechanism/observation/effect; Lisp owns
  interpretation/policy/protocol/derived meaning; TCP framing,
  retry/backoff, and routing are explicitly Lisp's job, not the host's.

Conclusion: a Python bridge is not a new architectural problem. It is
an ordinary Lisp protocol (like `knowledge.my`'s) running over transport
that already exists (`lib/tcp.my`), carried in a wire format that
already exists (`write-to-string`/`read`). **Reuse before invention.**

## Explicit architectural guards (unchanged, still apply)

- No `Value::Python`, `Value::NumPy`, or any per-runtime `Value`
  variant.
- No `match runtime == "python"`-style branching inside
  `crates/my-lisp`'s evaluator.
- No foreign library function (`numpy.linalg.eig`, `math.sqrt`, ...)
  ever enters Canon or `lib/surface/semantic-registry.wsm`.
- No foreign pointer/handle (Python `id()`, a bridge-local table index)
  is ever treated as my-lisp semantic identity.
- Rust is never the spec for a foreign API; the bridge script is a
  thin, auditable adapter (like `asm/nucleus.s` for wsm-my-lisp), not a
  general Python binding.
- No per-backend semantic corpus forks — `tests/fixtures/conformance.my`
  stays the one contract regardless of which runtime executes a call.
- Rust-code line count (added or deleted) is never the success metric.
  The metric is: **a new foreign runtime needs a new adapter, not a new
  evaluator branch and not new core semantics.**

## Revised architecture (v0: zero core changes)

```text
                    MY-LISP
                       |
                lib/foreign.my
          (foreign-import/member/call/release)
                       |
          canonical S-expression request/response
             (write-to-string / read, framed
              the same way lib/knowledge.my frames)
                       |
              tcp-connect / tcp-read / tcp-write
                  (already exist, unchanged)
                       |
                      TCP
                       |
              python-bridge.py
        (ref table: id -> live Python object,
         importlib.import_module, getattr, call)
                       |
       NumPy / SciPy / Astropy / PyTorch / ...
```

`crates/my-lisp` and `my-lisp-host` change: **zero.** Every new line of
code for v0 is either `lib/foreign.my` (Lisp) or `python-bridge.py`
(the foreign-side adapter, outside this repo's semantic surface
entirely, same status as `foreign_python_bridge.py` already is).

### Foreign object identity without any new `Value` variant

A foreign object is represented as ordinary serializable Lisp data —
an opaque reference, not a new value shape:

```lisp
(foreign-ref 42)
```

`python-bridge.py` holds the only table that matters: `id -> live
Python object`. My-lisp never sees a Python pointer, never needs `eq`
to understand foreign identity, and never needs a `Value::Foreign`
variant, because `(foreign-ref 42)` is just a two-element list —
`equal?` already compares it structurally, correctly, for free. If a
narrower comparison is ever wanted, it is an ordinary Lisp function
(`foreign-ref=?`), not a new primitive:

```lisp
(def foreign-ref=?
  (lambda (a b) (eq (second a) (second b))))
```

This retires Decision A, Decision B, and Decision C from the original
plan simultaneously: there is no foreign-value representation choice
to make in core (Decision A), the capability layer is confirmed
Lisp-owned with **no new raw primitives at all**, not even the four
minimal ones previously proposed (Decision B), and `eq` identity for
foreign objects is not a language question — it is bridge-table
bookkeeping the Lisp protocol layer can define however it wants
(Decision C).

### Wire protocol: an envelope over the existing canonical format

No new serializer, no JSON, no custom binary. A small envelope shape
riding on `write-to-string`/`read`, versioned by its own leading
symbol so a future incompatible change is a new symbol, not a break:

```lisp
(foreign/1 REQUEST-ID import "numpy")
(foreign/1 REQUEST-ID call (foreign-ref 12) "sqrt" (9))
(foreign/1 REQUEST-ID ok 3.0)
(foreign/1 REQUEST-ID error "ValueError" "math domain error")
```

Framing reuses `lib/knowledge.my`'s proven newline-delimited pattern
(`tcp-read-frame`): `write-to-string` already escapes embedded
newlines, so one envelope per line is an unambiguous frame boundary,
exactly as `exchange-knowledge-package` already relies on. No design
work remains here beyond picking the envelope shape above, which this
plan does now rather than deferring to a later phase — there is no
async/concurrency question to defer either, since one connection
carries one request at a time, matching the existing knowledge-exchange
protocol's own concurrency model (open a new connection per exchange
if you need concurrency, exactly as `lib/knowledge.my` already does).

## Task sequence (replaces the old 8-phase structure)

```text
POLYGLOT-SIMPLE-1  Python bridge script over existing TCP transport
        |
POLYGLOT-SIMPLE-2  lib/foreign.my (import/member/call/release), pure Lisp
        |
POLYGLOT-SIMPLE-3  math.sqrt end-to-end witness
        |
POLYGLOT-SIMPLE-4  One real NumPy operation through the same unchanged API
```

Manual bridge startup (`python python-bridge.py`) is accepted for v0 —
automatic process spawning is explicitly **not** a v0 requirement. If
manual startup proves a real friction point in practice, that
observation is the earned justification for a `process-spawn-raw`
host primitive later — not before.

Likewise, NumericBuffer/shared-memory for large NumPy arrays is not
designed now. Small values travel the canonical S-expression control
plane above; a high-bandwidth data plane is only justified after a
real NumPy workload is benchmarked and found too slow over text TCP.
Control plane and data plane are architecturally separate concerns and
must not be conflated:

```text
CONTROL PLANE  — S-expressions over TCP — cheap, universal, done above
DATA PLANE     — shared memory / NumericBuffer — only if/when earned
```

## What was explicitly retired from the previous plan version

The previous Phase 0-8 structure (comparison tables for
`Value::Foreign` vs. generic opaque resource vs. Lisp-only reference;
four new `foreign-*-raw` host primitives; a from-scratch wire-protocol
design comparing S-expression/JSON/binary) is retired, not because it
was wrong to consider, but because the audit shows the comparison
already has a clear winner using code this repository already runs in
production for other capabilities. The general backend-neutral-oracle
generalization work (previously "Phase 7") is unaffected by this
revision and continues to track independently — it has no dependency
on the foreign-runtime track.

## Non-goals (unchanged)

- Julia/JS/R adapters ahead of a real, concrete need.
- SciPy/Astropy/PyTorch before the basic Python adapter
  (`POLYGLOT-SIMPLE-3`/`4`) is proven.
- Any core-crate change. If a `POLYGLOT-SIMPLE-*` task turns out to
  need one, that need itself must be written down and justified before
  writing the code, per this repository's own design-fixed-before-code
  discipline — it is not expected going in.

## Rollback / fail-closed conditions

- If `lib/foreign.my`'s protocol cannot be expressed without a core
  change, stop and write down exactly what is missing and why
  `lib/tcp.my`/`write-to-string` are insufficient, before touching
  `crates/my-lisp`.
- If the manual-bridge-startup friction or the text-TCP performance
  ceiling is hit, that becomes a new, separately justified task
  (`process-spawn-raw` or a data-plane design) — not a silent scope
  creep inside `POLYGLOT-SIMPLE-*`.

## What can be deleted after this lands

Nothing needs deleting — `foreign_python_probe.rs`/
`foreign_python_bridge.py` remain as the standalone transport-feasibility
evidence they always were; `python-bridge.py` for `POLYGLOT-SIMPLE-1`
is a new, separate script built for the real protocol, not a
replacement for the PoC's narrower one.
