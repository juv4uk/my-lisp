# `Value::Builtin` identity migration map (research, not implementation)

## What happened and why this document exists instead of a code fix

Auditing my-lisp for duplicated primitives found a real, previously
undetected gap: `impl PartialEq for Value` had no match arm for
`Value::Builtin` at all, so `(eq car car)` returned `()` — no
first-class builtin was ever `eq` to itself, silently contradicting
`crates/my-lisp/src/eval/canon.rs`'s own
`three_callable_surfaces_share_one_stable_handle` test (which proves
`car`/`перше`/`ādi` share one `Rc` at construction, but nothing in the
language could observe that sharing).

The first response to this finding added `(Value::Builtin(left),
Value::Builtin(right)) => Rc::ptr_eq(left, right)` — the same pattern
already used for `Closure`/`Macro`/`TcpConnection`. **That fix was
reverted before committing.** The owner's correction: this repo has
been deliberately moving *away* from treating `Value::Builtin` (a Rust
runtime object) as part of language identity, toward numeric semantic
ID as the identity a surface spelling resolves to — a Rust object is
one *implementation projection* of that ID, not the identity itself.
Adding `Rc::ptr_eq` as the fix would have re-anchored `eq`'s meaning to
a Rust pointer at exactly the point the ecosystem is trying to make
that pointer replaceable (by a CML-compiled value, an FPGA opcode, a
pure-Lisp definition) without changing what `(eq car car)` means.

`docs/PLAN-FULL-LANGUAGE-PARITY.md`'s own Invariant 4 and Gate B
previously *required* `Rc::ptr_eq` as the acceptance proof — corrected
in the same commit as this document, so a future agent doesn't
rediscover this bug and get led toward the same reversed fix by an
outdated plan.

## The four found regression cases — kept, reframed

These are real, verified observations (see the live-tested transcript
below) and remain the target behavior. What changes is *why* they
should hold:

```
(eq car car)     ; must be t  -- same semantic ID (0005), not "same Rc"
(eq car перше)    ; must be t  -- as above: same ID across surfaces
(eq car ādi)     ; must be t  -- as above
(eq car cdr)     ; must be () -- different semantic IDs (0005 vs 0006)
```

Live-verified today (current, unfixed runtime — recorded here as the
baseline this migration starts from, not as a claim these already
pass):

```
$ echo '(eq car car)' | my-lisp
()
```

No test asserting these currently exists in the tree (checked:
`crates/my-lisp/tests/first_class_builtins.rs` has none of the four).
They should be added **once** the underlying representation exists to
make them true for the right reason — adding them now, backed by
`Rc::ptr_eq`, would be exactly the trap the owner named: "regression
tests, documentation, and dependencies will start defending the
existence of `Builtin`."

## Dependency map: everywhere `Value::Builtin` / `Rc<Builtin>` / `Rc::ptr_eq`-for-callables appears today

Found by direct grep across `crates/*/src/`, not assumed:

| File | Line(s) | What it does with `Builtin` | Migration weight |
|---|---|---|---|
| `crates/my-lisp/src/value.rs` | struct def (`Builtin { name, func }`), `Value::Builtin(Rc<Builtin>)` variant | Owns the type itself | **Core** — any redesign starts here |
| `crates/my-lisp/src/eval/canon.rs` | `materialize_value`, `build_value_registry`, `CANON_VALUES` thread-local | Constructs one `Value::Builtin` per Canon identity (`PRIM_CAR` etc.), keyed by `CanonicalIdentity` — **already numeric-ID-shaped internally**, just not exposed that way to `eq` | **Low** — this is the closest existing code to the target shape; the numeric key (`CanonicalIdentity`, which is 1:1 with a semantic-registry ID) already exists here, it just isn't consulted by `PartialEq for Value` |
| `crates/my-lisp/src/eval/builtins.rs` | `install()`, `define!` macro | Constructs ordinary (non-Canon) builtins (`+`, `abs`, etc.) as one `Value::Builtin` each, bound once per name in the root environment | **Medium** — these builtins have no numeric semantic ID *requirement* the way Canon does (semantic-registry IDs like `0104` for `+` exist but nothing currently reads them at construction time here) |
| `crates/my-lisp/src/eval/mod.rs` | line ~100 (`apply`), line ~222 | The actual call-dispatch: `Value::Builtin(builtin) => (builtin.func)(...)` | **Core** — this is "apply," the mechanism a semantic-ID-indirected design would need to change: `apply(id, args)` → look up implementation projection → invoke, instead of `apply` closing directly over a Rust closure |
| `crates/my-lisp/src/eval/closures.rs` | line ~348 | Treats `Value::Builtin` as one case of "a callable value" alongside `Closure`, for whatever generic callable-handling that function does | **Medium** |
| `crates/my-lisp/src/eval/macro_substrate.rs` | line ~11 | Constructs a `Value::Builtin` directly (the narrow `make-macro` primitive) | **Low** — one specific, narrow construction site |
| `crates/my-lisp/src/eval/special_forms.rs` | doc comment only | References the pattern in prose, no code dependency | **None** |
| `crates/my-lisp/src/language_items.rs` | line ~408, ~435 | Introspection: builds the `language_items()` metadata table (used by LSP arity diagnostics, help) by matching on `Value::Builtin` | **Medium** — tooling-facing, would need the same ID-indirection if `Builtin` stops being the thing inspected |
| `crates/my-lisp/src/layout.rs` | line ~77 | NaN-boxing: `Value::Builtin(b) => NanBox(pack_ptr(TAG_PRIMITIVE, Rc::as_ptr(b) as u64))` — packs the **raw `Rc` pointer** into the compact FFI-style value representation under `TAG_PRIMITIVE` | **High, not low** — corrected after reading it directly: this is the same Rust-pointer-as-identity leak as the reverted `eq` fix, but at the memory-layout level. Any external consumer of this NaN-boxed representation (the same family of concern as `wsm-my-lisp`'s `Tag::Boxed` work) would see a raw pointer, not a semantic ID — a pointer that is meaningless outside this process and changes every run. If semantic-ID-based identity becomes real, this tag's payload is exactly the kind of thing that would need to carry an ID instead of (or alongside) a pointer. |
| `crates/my-lisp/src/presentation.rs` | line ~88 | Printing: `#<builtin name>` rendering | **Low** — printing can key off whatever identity representation wins; trivial to adapt either way |

## What a real migration would need (not designed here — scope note)

The owner's sketch (`Value::SemanticRef(0005)` replacing
`Value::Builtin(Rc<dyn Fn>)`, with `apply` resolving the ID to an
implementation projection) is a genuine language-core redesign, not a
bug fix:

- `apply`'s dispatch (`eval/mod.rs`) would need an ID→implementation
  lookup table instead of calling `builtin.func` directly — this table
  is conceptually close to `canon.rs`'s existing `CANON_VALUES`
  thread-local, generalized past just the seven Canon identities.
- Ordinary (non-Canon) builtins (`+`, `abs`, library functions) don't
  currently have a numeric ID *required* at construction — the
  semantic-registry entry exists (e.g. `0104` for `+`) but nothing
  reads it when `builtins.rs::install()` runs. Deciding whether every
  builtin gets a mandatory ID, or only ones that need cross-surface
  `eq` identity, is a real open design question this map does not
  resolve.
- Tooling (`language_items.rs`, LSP arity diagnostics, printing) reads
  `Value::Builtin` directly today; each would need to either read
  through the new indirection or keep reading a still-present
  `Value::Builtin` that's now *one possible* implementation projection
  rather than the only one.
- This is explicitly **not** a rename (`Builtin` → `Primitive` with the
  same shape) — the owner named that exact trap. The actual change is
  structural: identity moves from "which Rust object" to "which
  numeric ID," and the Rust object becomes retrievable *from* the ID,
  not the other way around.

## What was NOT done, on purpose, per this session's instruction

- No `Rc::ptr_eq` arm was added or committed for `Value::Builtin`.
- No mechanical rename of `Builtin` → anything else.
- No `Value::SemanticRef` or equivalent was implemented — this is a
  research/mapping document only, per the explicit instruction to map
  dependencies before designing the replacement.
- `crates/my-lisp/tests/first_class_builtins.rs` was left unchanged —
  the four regression cases above are recorded here, not committed as
  tests, so they don't start "defending the existence of `Builtin`"
  before the real design lands.
