# Host-primitive-dispatch conformance fixtures — my-lisp-cyberpunk

Written for `wsm-my-lisp`'s step 3 (minimal reader+evaluator for
one-shot console commands, no closures — per the earlier agreed MVP
scope) and `cml`'s optional codegen role. Every value below was
produced by running the actual `my-lisp` CLI (`./target/debug/my-lisp.exe`,
`v0.38.0`), not written from memory — this is an oracle, not prose.
`(def X ...)` lines simulate what the host side would register before
a console command runs; they are not part of my-lisp's own contract,
just the fixture setup.

Per the owner's standing instruction, this project programs in
Ukrainian by default (`lib/surface/uk.my`, 100% Ukrainian Surface
Coverage) — so the fixtures below use Ukrainian identifiers for the
invented host-command names (`телепортуй`, `дай-зброю`,
`збережи-гру`), not English placeholders. Ukrainian identifiers are
ordinary symbols to the reader/evaluator — verified directly below,
not assumed.

## 1. Reader fixtures (text → structure)

| Input | Read result (printed) |
|---|---|
| `(телепортуй гравець 100 200 -10)` | `(телепортуй гравець 100 200 -10)` — negative numbers are ordinary tokens, no special reader rule; Cyrillic identifiers read exactly like Latin ones |
| `(дай-зброю "пістолет" 5)` | `(дай-зброю "пістолет" 5)` — string literal (Cyrillic content) keeps its quotes in the reader's own printed form |
| `(збережи-гру)` | `(збережи-гру)` — zero-argument call, still a 1-element list |
| `(quote ())` | `()` — the empty list prints as itself, not `nil`/`NIL` |
| `(телепортуй гравець (+ x 10) y z)` | `(телепортуй гравець (+ x 10) y z)` — nested list as an argument reads as an ordinary sub-list, no special-casing |

No dotted-pair literals, no `'x` quote-sugar needed for any of these —
confirms your reader scope is sufficient for this fixture set.

## 2. Evaluator fixtures — host-primitive dispatch, no closures

Setup once per row (host pre-registers bindings — this is what "a
bare symbol argument must be bound via `def`" from my earlier answer
looks like concretely):

```lisp
(def гравець 42)
(def x 5)
(def телепортуй (lambda (хто x y z) (list хто x y z)))
(def дай-зброю (lambda (назва кількість) (list назва кількість)))
```

| Expression | Evaluated arguments | Result |
|---|---|---|
| `(телепортуй гравець 100 200 -10)` | `(42 100 200 -10)` | `(42 100 200 -10)` |
| `(дай-зброю "пістолет" 5)` | `("пістолет" 5)` | `("пістолет" 5)` |
| `(телепортуй гравець (+ x 10) 200 -10)` | `(42 15 200 -10)` | `(42 15 200 -10)` — **the `(+ x 10)` argument is evaluated before the call, not passed as a literal list** |

The third row directly answers cml's arity/nesting question: arguments
are not restricted to bare literals/symbols — an argument that is
itself a call (`(+ x 10)`) evaluates recursively before the outer
primitive is invoked, exactly like any other my-lisp function call.
This is ordinary evaluation order, not a host-dispatch-specific rule.

## 3. `cond` fixture (special form, confirmed against `language-contract.my`)

```lisp
(cond ((eq гравець 42) (quote відомий-гравець)) (t (quote невідомий-гравець)))
```
→ `відомий-гравець`. `t` is always truthy; the number `0` is also
truthy (unlike C/Python/JS) — only `Nil` and `Bool(false)` are falsy.
See `docs/language-core-axioms.md`'s G8 note if a fixture ever needs
to prove this specifically.

## 4. `UnknownSymbol` fixture — exact text, not just "an error"

```lisp
(збережи-гру)
```
with `збережи-гру` **not** pre-registered via `def`:

```
unknown symbol · nevidomyi symvol · unbekanntes Symbol: збережи-гру
```

Verbatim from `crates/my-lisp/src/eval/mod.rs`'s own error construction
(trilingual by design, not a formatting accident — keep the exact
`·`-separated three-language string if you want byte-for-byte
compatibility, or just match on the `збережи-гру`-suffix if that's
more practical for a minimal implementation). Verified directly: the
error text embeds the offending Cyrillic identifier unchanged, no
transliteration or mangling.

## String representation — resolved (2026-09-10, superseded once, now final)

The open question from earlier ("wsm-my-lisp's tagged-word ABI has no
String type yet") is resolved and shipped, not merely proposed.
History, briefly, because the final tag value changed once for a good
reason:

1. First cut: `TAG_STRING` at tentative value `7` (the last free slot
   in the 3-bit Tag), backed by a non-interning append-only
   `StringTable` (offset+length into a UTF-8 arena) — mirroring the
   existing `SymbolTable` pattern but without deduplication, matching
   my-lisp's own semantics exactly (strings compare structurally via
   `equal?`, not identity via `eq?`, so identical literals are
   legitimately independent allocations, unlike interned symbols).
2. Before that tag value was ratified in `wsm-target-contract`, this
   doc flagged a real risk: `TAG_STRING=7` would consume the last free
   3-bit tag slot, and my-lisp's own `Value` enum
   (`crates/my-lisp/src/value.rs`) still has `Vector`, `NumericBuffer`,
   and TCP-handle variants with no tagged-word representation at all —
   spending the last slot on one of several future needs risked an
   ABI migration later.
3. cml's engineering review (asked because cml generates code against
   this exact ABI) recommended against widening `TAG_BITS` 3→4 (an
   expensive, one-way change without proven need) and proposed
   `TAG_BOXED=7` instead — one generic tag for String/Vector/
   NumericBuffer/TCP-handles together, with an internal discriminant
   byte inside the boxed object itself. `Rational`/`Macro`/`Bool` need
   no primary tag (already covered by other patterns); `Builtin`/
   `Closure` are correctly absent from this list — first-class
   callables are out of scope for the current Cyberpunk MVP
   (host-primitive dispatch by symbol, no closures).
4. wsm-my-lisp implemented `TAG_BOXED=7` per that recommendation.
   Commit `09d7c20`, 36/36 tests passing. `(дай-зброю "пістолет" 5)`
   from §2 round-trips end-to-end through the real asm nucleus with
   this final representation, not just the Rust reference.

No further action needed on this thread — the tag-space-exhaustion
risk that motivated the change is now closed by construction (one
boxed tag absorbs future variants via discriminant byte, not by
consuming additional primary tag bits).

## Also-valid English forms (equivalent, not preferred)

The English-named forms (`teleport`, `give-weapon`, `save-game`) work
identically — my-lisp's evaluator does not distinguish identifier
scripts — but Ukrainian is this project's default per the owner's
standing instruction, so treat the section above as the canonical
fixture set and this note only as a compatibility footnote if an
English-only test harness is more convenient for a given step.

## Scope note

This fixture set intentionally does not include `lambda`/closures
(out of MVP scope per the earlier agreed division), macros, or
multi-clause `cond` beyond what's shown in §3 — those are already
covered generally by `tests/fixtures/conformance.my`'s own `cond`
fixtures if a future slice needs them. Ask if a specific case here
turns out to be insufficient once real dispatch code is written against
it — this is meant to be a living oracle, not a one-shot handoff.
