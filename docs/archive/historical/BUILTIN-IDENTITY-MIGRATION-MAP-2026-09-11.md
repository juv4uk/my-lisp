# `Value::Builtin` identity migration map — historical pre-#95 baseline

> **Archived 2026-09-13.** This document records the repository state *before* PR #95 introduced `Value::SemanticRef` for Canon callables. Statements below such as “current, unfixed runtime”, “no `Value::SemanticRef`”, raw `TAG_PRIMITIVE` pointer payloads, and the old `CANON_VALUES` construction describe that historical baseline only. They are preserved as design evidence, not current architecture.

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

Historical RED baseline, verified before PR #95:

```
$ echo '(eq car car)' | my-lisp
()
```

No test asserting these existed in the tree at that point
(`crates/my-lisp/tests/first_class_builtins.rs` had none of the four).
They were intentionally deferred until the underlying representation
could make them true for the right reason rather than via `Rc::ptr_eq`.

## Dependency map: everywhere `Value::Builtin` / `Rc<Builtin>` / `Rc::ptr_eq`-for-callables appeared at the time

Found by direct grep across `crates/*/src/`, not assumed:

| File | Line(s) | What it did with `Builtin` | Migration weight |
|---|---|---|---|
| `crates/my-lisp/src/value.rs` | struct def (`Builtin { name, func }`), `Value::Builtin(Rc<Builtin>)` variant | Owned the type itself | **Core** — any redesign started here |
| `crates/my-lisp/src/eval/canon.rs` | `materialize_value`, `build_value_registry`, `CANON_VALUES` thread-local | Constructed one `Value::Builtin` per Canon identity (`PRIM_CAR` etc.), keyed by `CanonicalIdentity` | **Low** — closest existing code to the target shape |
| `crates/my-lisp/src/eval/builtins.rs` | `install()`, `define!` macro | Constructed ordinary (non-Canon) builtins (`+`, `abs`, etc.) as one `Value::Builtin` each | **Medium** |
| `crates/my-lisp/src/eval/mod.rs` | line ~100 (`apply`), line ~222 | Call-dispatch: `Value::Builtin(builtin) => (builtin.func)(...)` | **Core** |
| `crates/my-lisp/src/eval/closures.rs` | line ~348 | Treated `Value::Builtin` as one callable case alongside `Closure` | **Medium** |
| `crates/my-lisp/src/eval/macro_substrate.rs` | line ~11 | Constructed a `Value::Builtin` directly (`make-macro`) | **Low** |
| `crates/my-lisp/src/eval/special_forms.rs` | doc comment only | Referenced the pattern in prose | **None** |
| `crates/my-lisp/src/language_items.rs` | line ~408, ~435 | Introspection matched on `Value::Builtin` | **Medium** |
| `crates/my-lisp/src/layout.rs` | line ~77 | NaN-boxed the raw `Rc` pointer under `TAG_PRIMITIVE` | **High** |
| `crates/my-lisp/src/presentation.rs` | line ~88 | Printed `#<builtin name>` | **Low** |

## What a real migration would need (historical design sketch)

The owner's sketch (`Value::SemanticRef(0005)` replacing
`Value::Builtin(Rc<dyn Fn>)`, with `apply` resolving the ID to an
implementation projection) was identified as a genuine language-core
redesign rather than a bug fix:

- `apply`'s dispatch (`eval/mod.rs`) would need an ID→implementation
  lookup table instead of calling `builtin.func` directly.
- Ordinary (non-Canon) builtins (`+`, `abs`, library functions) did not
  yet require a numeric ID at construction even when the semantic
  registry had an entry.
- Tooling (`language_items.rs`, LSP arity diagnostics, printing) read
  `Value::Builtin` directly and would need to either read through the
  new indirection or keep treating `Builtin` as a host mechanism only.
- This was explicitly **not** a rename (`Builtin` → `Primitive` with the
  same shape). The intended structural change was identity moving from
  “which Rust object” to “which numeric ID”, with the Rust object an
  implementation projection.

## What was NOT done in the research commit

- No `Rc::ptr_eq` arm was added or committed for `Value::Builtin`.
- No mechanical rename of `Builtin` → anything else.
- No `Value::SemanticRef` or equivalent was implemented **in that research commit**.
- `crates/my-lisp/tests/first_class_builtins.rs` was left unchanged **in that research commit**.

## What changed later

PR #95 completed the first real vertical migration for Canon callables:

- Canon callable values now materialize as `Value::SemanticRef(id)`.
- `eq` compares Canon callables by semantic ID, so admitted peer surfaces for ID `0005` are observably identical while `0005` and `0006` remain distinct.
- evaluator application resolves the semantic reference to the current implementation projection.
- `TAG_PRIMITIVE` carries the numeric semantic ID; legacy host-only builtin pointers use a separate host tag.
- first-class builtin regression tests cover the cross-surface identity witness.
- unknown semantic callable IDs fail closed.

Legacy non-Canon `Value::Builtin` remains an implementation mechanism where appropriate; that does not make the Rust object the semantic identity.