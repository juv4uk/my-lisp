# Multi-host capability timing — design note (CP-MULTI-HOST-CAPABILITY-MODEL)

Written before a second host-specific capability set exists, per this
repo's own task recommendations (`docs/cyberpunk-task-recommendations-2026-09-10.md`,
priority 8.5) and the tension flagged in `docs/cyberpunk-paradigm-fit.md`.
Grounded directly in the real registry code
(`crates/my-lisp/src/eval/capabilities.rs`), not speculation about what
it might do.

## The actual mechanism, as it exists today

```rust
pub type HostFn = fn(&[Expr], &Environment, Span) -> Result<Value, LanguageError>;
```

One fact settles this whole design note: `HostFn` is a plain,
synchronous function pointer. It runs to completion on the calling
thread and returns a `Value` directly. There is no timeout parameter,
no cancellation token, no yield point, no async boundary anywhere in
the signature or in `dispatch_capability`'s call site. This is not an
oversight — `crates/my-lisp-host` genuinely registers blocking
capabilities against exactly this signature: `tcp-accept` blocks until
a connection arrives; `process-run-raw` blocks until the child process
exits; `read-file` blocks on disk I/O. The native CLI host is allowed
to block, so the mechanism correctly never had to express "how long is
too long."

## Why this is fine for two hosts and not obviously fine for a third

- **Native CLI** (`my-lisp-host`): blocking is the correct, unremarkable
  behavior. A `tcp-accept` capability that blocks the whole process
  until a connection arrives is exactly what a CLI user wants.
- **WASM**: today's WASM embedding (per `lib.rs`'s `load_core_library`
  et al.) does not install the process/TCP capabilities at all — it
  simply never calls those particular `register_capability` calls. The
  timing question is dodged by omission, not answered, which has been
  fine because nothing WASM-hosted has needed those capabilities yet.
- **Cyberpunk (CET/RED4ext)**: a game engine's per-frame budget is a
  real constraint this ecosystem has not had to satisfy before. CET's
  Lua bridge and RED4ext's native tick both run inside a frame loop
  where a capability that blocks for, say, 50ms causes a visible
  stutter or a watchdog-triggered hang — a qualitatively different
  failure mode than "the CLI process pauses for a moment," which is
  merely mildly slow.

The `запиши-лог` capability proven in the real in-game smoke test
(`my-lisp-cyberpunk#1`) is a single synchronous log write — well inside
any frame budget, which is exactly why it was chosen as the first
vertical slice. It does **not** exercise this tension at all; it was
deliberately picked to avoid it, not to resolve it.

## What breaks first, concretely

The moment a *second* Cyberpunk capability is proposed that is not
"instant" — e.g. a `player-position/read` that must marshal through
RED4ext's RTTI reflection layer, or any capability that touches game
state guarded by a lock the render thread also wants — the current
`HostFn` signature gives the adapter no structural way to say "this
must complete within N microseconds or abort," and no way for the
evaluator side to know it should treat this capability differently
from a CLI-side `read-file`. The adapter would have to enforce a
budget *internally*, catching its own possible overrun before ever
returning a `Value` — my-lisp's evaluator has no visibility into or
enforcement of that constraint today, and nothing in the current
design asks it to.

## Options, not a decision

This note deliberately stops short of choosing one, per the ecosystem's
own "first step belongs to whoever has the real need" discipline — no
frame-budgeted capability has been requested yet, so committing to a
mechanism now would be solving a hypothetical.

1. **Do nothing until it's needed.** The adapter enforces its own
   internal timeout/abort and always returns *something* (an error
   Value on overrun) within `HostFn`'s existing synchronous contract.
   Zero changes to `crates/my-lisp`. Works as long as every future
   Cyberpunk capability can be made to fit a "always returns fast or
   returns an error fast" shape — plausible for read-only query
   capabilities, the only kind currently in scope per
   `my-lisp-cyberpunk#1`'s own "not yet: mutation, callbacks" boundary.
2. **A capability-declared budget hint.** Extend the registration API
   (not `HostFn`'s call signature) with an optional per-capability
   "expected to complete within" metadata the evaluator can expose to
   diagnostics — advisory only, not enforced, so it costs nothing at
   the call site and doesn't change `HostFn`'s type.
3. **A real async/cooperative-yield capability protocol.** Only
   relevant if a genuinely long-running or blocking-on-game-event
   capability is ever needed (arguably indistinguishable from wanting
   closures/callbacks, which is explicitly out of scope for the
   current MVP per the paradigm-fit doc's own "next request" warning).
   This is a real language-core change, not a host-adapter detail, and
   should not be attempted speculatively.

## Recommendation, if one is needed today

Option 1, with option 2 as a cheap, non-invasive addition if
diagnostics turn out to need it. Nothing in `my-lisp-cyberpunk#1`'s
current or near-term scope (read-only query capabilities) requires
more than that, and inventing a budget-enforcement mechanism in the
capability-free core before a real capability needs it would violate
this ecosystem's own "owner's first step" discipline the same way a
speculative `repo.my` would have.
