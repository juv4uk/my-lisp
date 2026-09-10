# Opaque capability-value semantics for future RTTI reads — research note

Prep work, not implementation, per my-lisp-cyberpunk's own framing:
"дай семантичне визначення/фікстури, коли wsm-my-lisp прийде з
конкретним питанням; поки просто будь готовий." No code changes here
— this documents an existing precedent so the answer is ready when a
concrete `game-version`/`player-position/read` proposal arrives.

## The precedent already exists: TCP handles

my-lisp already has exactly this shape of problem, solved, in
`crates/my-lisp/src/value.rs`:

```rust
TcpConnection(Rc<RefCell<TcpStream>>),
TcpListener(Rc<TcpListener>),
```

Three properties, verified directly in the source, that answer
Cyberpunk's question by direct analogy:

1. **Opaque, not raw.** Lisp code never sees a `TcpStream`/socket file
   descriptor/pointer value. It sees a `Value::TcpConnection`, a
   first-class but *opaque* handle. There is no primitive that unwraps
   it to expose the underlying pointer/fd to a Lisp program — every
   operation on it goes through a named host capability
   (`tcp-read-raw`, `tcp-write-raw`, `tcp-close`, …), never direct
   field access.
2. **Identity equality, not structural.** `value.rs:541-542`:
   `(Value::TcpConnection(left), Value::TcpConnection(right)) =>
   Rc::ptr_eq(left, right)`. Two handles are `eq?` only if they are
   literally the same handle, never by comparing what they point to —
   documented in the source's own comment as "the same rule as
   identity — it is a resource handle, not a structurally-comparable
   value," same category as `Closure`/`Macro`.
3. **Never Lisp-constructible.** There is no `(make-tcp-connection
   ...)` primitive. The only way a program acquires one is by calling
   a host capability that returns it (`tcp-connect`). A Lisp program
   cannot forge, fabricate, or synthesize a handle value out of other
   data — it can only receive one from a capability call, or receive
   `()`/an error if the call fails.

## Direct mapping to a future `GameHandle`

If/when wsm-my-lisp or my-lisp-cyberpunk proposes a capability like
`game-version` or `player-position/read`, the semantic answer (from
my-lisp's side, before any machine-ABI tag number is involved) is:

- **`game-version` needs no opaque handle at all.** If it returns a
  plain value (a string, a list of three integers), it's an ordinary
  capability like `read-file` — no new Value variant needed, already
  covered by the existing `HostFn` mechanism.
- **`player-position/read` returning a live, mutable game-object
  reference** (as opposed to a snapshot value) *would* need an opaque
  handle, following the `TcpConnection` precedent exactly: a new
  `Value` variant (e.g. `Value::GameHandle(Rc<...>)`), identity-only
  equality, no Lisp-level unwrap primitive, only acquirable by calling
  a registered capability. Read operations on it (e.g. `(rtti-get
  handle "position")`) are themselves separate registered
  capabilities, not primitives on the Value type — exactly how
  `tcp-read-raw` is a capability operating *on* a `TcpConnection`
  value, not a method the Value type itself exposes.
- **If the position is just read once and returned as plain numbers**
  (the simpler, MVP-appropriate shape — "read the position, get three
  numbers back"), no handle is needed at all; this is the
  recommended shape for the *first* capability of this kind, matching
  the existing `запиши-лог` precedent of picking the simplest
  possible vertical slice.

## Read vs. mutate — how the boundary is actually enforced

Per capability-model design already written
(`docs/cyberpunk-multi-host-capability-model.md`): my-lisp's capability
registry has no built-in read/write permission concept at the type
level. The read/mutate boundary is enforced entirely by *which
capabilities the adapter chooses to register* — if the adapter never
registers a `player-position/write` or `inventory/give`, no Lisp
program can mutate anything, regardless of what handles exist. This
matches `my-lisp-cyberpunk#1`'s own stated boundary ("first capability
must not mutate save/inventory") and needs no new language-level
permission mechanit — the enforcement point is the adapter's own
registration code, which already lives entirely in that repo per the
established dependency boundary (semantics → my-lisp, live
runtime/FFI → wsm-my-lisp, game-specific code → my-lisp-cyberpunk).

## What a future fixture set would need (not written yet)

When a concrete capability is proposed, the fixture set should cover,
by direct analogy to the TCP handles' own test coverage:
1. A successful capability call returning an opaque handle, printed
   form (`<game-handle>` or similar, matching `<tcp-connection>`'s own
   pattern in `presentation.rs`).
2. `(eq? h h)` → `t` for the same handle, `(eq? h1 h2)` → `nil` for two
   handles from two separate calls, even if they conceptually refer to
   "the same" game object — same identity-not-structural rule as TCP.
3. Attempting to use a handle after the capability that produced it
   would consider it invalid (session ended, object destroyed) — my
   lisp's own answer here should follow whatever `tcp-read` does on
   read of a closed connection, once a concrete failure mode is named
   by the adapter side.

Not building any of this now — recorded so the answer is ready the
moment wsm-my-lisp asks a concrete question, per the request that
started this note.
