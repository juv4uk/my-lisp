# Host-primitive-dispatch conformance fixtures — my-lisp-cyberpunk

Written for `wsm-my-lisp`'s step 3 (minimal reader+evaluator for
one-shot console commands, no closures — per the earlier agreed MVP
scope) and `cml`'s optional codegen role. Every value below was
produced by running the actual `my-lisp` CLI (`./target/debug/my-lisp.exe`,
`v0.38.0`), not written from memory — this is an oracle, not prose.
`(def X ...)` lines simulate what the host side would register before
a console command runs; they are not part of my-lisp's own contract,
just the fixture setup.

## 1. Reader fixtures (text → structure)

| Input | Read result (printed) |
|---|---|
| `(teleport player 100 200 -10)` | `(teleport player 100 200 -10)` — negative numbers are ordinary tokens, no special reader rule |
| `(give-weapon "pistol" 5)` | `(give-weapon "pistol" 5)` — string literal keeps its quotes in the reader's own printed form |
| `(save-game)` | `(save-game)` — zero-argument call, still a 1-element list |
| `(quote ())` | `()` — the empty list prints as itself, not `nil`/`NIL` |
| `(teleport player (+ x 10) y z)` | `(teleport player (+ x 10) y z)` — nested list as an argument reads as an ordinary sub-list, no special-casing |

No dotted-pair literals, no `'x` quote-sugar needed for any of these —
confirms your reader scope is sufficient for this fixture set.

## 2. Evaluator fixtures — host-primitive dispatch, no closures

Setup once per row (host pre-registers bindings — this is what "a
bare symbol argument must be bound via `def`" from my earlier answer
looks like concretely):

```lisp
(def player 42)
(def x 5)
(def teleport (lambda (who x y z) (list who x y z)))
(def give-weapon (lambda (name count) (list name count)))
```

| Expression | Evaluated arguments | Result |
|---|---|---|
| `(teleport player 100 200 -10)` | `(42 100 200 -10)` | `(42 100 200 -10)` |
| `(give-weapon "pistol" 5)` | `("pistol" 5)` | `("pistol" 5)` |
| `(teleport player (+ x 10) 200 -10)` | `(42 15 200 -10)` | `(42 15 200 -10)` — **the `(+ x 10)` argument is evaluated before the call, not passed as a literal list** |

The third row directly answers cml's arity/nesting question: arguments
are not restricted to bare literals/symbols — an argument that is
itself a call (`(+ x 10)`) evaluates recursively before the outer
primitive is invoked, exactly like any other my-lisp function call.
This is ordinary evaluation order, not a host-dispatch-specific rule.

## 3. `cond` fixture (special form, confirmed against `language-contract.my`)

```lisp
(cond ((eq player 42) (quote known-player)) (t (quote unknown-player)))
```
→ `known-player`. `t` is always truthy; the number `0` is also truthy
(unlike C/Python/JS) — only `Nil` and `Bool(false)` are falsy. See
`docs/language-core-axioms.md`'s G8 note if a fixture ever needs to
prove this specifically.

## 4. `UnknownSymbol` fixture — exact text, not just "an error"

```lisp
(save-game)
```
with `save-game` **not** pre-registered via `def`:

```
unknown symbol · nevidomyi symvol · unbekanntes Symbol: save-game
```

Verbatim from `crates/my-lisp/src/eval/mod.rs`'s own error construction
(trilingual by design, not a formatting accident — keep the exact
`·`-separated three-language string if you want byte-for-byte
compatibility, or just match on the `save-game`-suffix if that's more
practical for a minimal implementation).

## Scope note

This fixture set intentionally does not include `lambda`/closures
(out of MVP scope per the earlier agreed division), macros, or
multi-clause `cond` beyond what's shown in §3 — those are already
covered generally by `tests/fixtures/conformance.my`'s own `cond`
fixtures if a future slice needs them. Ask if a specific case here
turns out to be insufficient once real dispatch code is written against
it — this is meant to be a living oracle, not a one-shot handoff.
