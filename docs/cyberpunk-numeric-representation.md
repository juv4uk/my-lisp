# Host float → exact Rational at the boundary — revised recommendation

Supersedes the numeric-representation part of my earlier answer (which
recommended `Value::Number(f64, Exactness::Inexact)` for RTTI floats
like player position). Per the owner's direct standing decision,
relayed via my-lisp-cyberpunk: **"РАЦІОНАЛЬНІ ЧИСЛА ЦЕ НАША ФІШКА"** —
exact rationals are this ecosystem's identity, not a fallback float
type, and this applies to host-boundary numbers too, not only numbers
a program computes internally. This is consistent with the wider
ecosystem philosophy already established (esp32-pendulum: "exact
rational arithmetic, no float/double in the pipeline") and with
`lib/rational.my`'s own presence in this repo as groundwork for exactly
this kind of need, not orphaned code.

## Correction to my earlier reasoning

My earlier answer treated "exact rational" and "IEEE754 float" as two
different *kinds* of information, where converting a hardware float to
a rational risked inventing false precision. That framing was wrong in
a specific, checkable way: **every finite `f64` already has an exact,
loss-free rational value** — mantissa × 2^exponent, always a ratio of
integers. There is no approximation step required to go from "a
float" to "an exact rational that equals it precisely." The real
design question was never "can we do this exactly," it was "which
exact rational" — the float's exact binary value (technically precise
but produces ugly, unreadable fractions), or the float's shortest
round-trip decimal representation converted exactly (matches what a
human/game designer would read off the number, at the cost of
technically not being the literal bit pattern).

## The mechanism already exists in this codebase

`crates/my-lisp/src/value.rs`'s `Rational::from_decimal_literal(text:
&str)` is the exact function this needs — it is already how my-lisp's
own reader turns a human-typed literal like `123.456` into a precise
`Rational` (mantissa over a power of ten, arbitrary-precision `BigInt`
numerator/denominator, not `f64`-bounded). The recommended conversion
path for a host float:

```
let text = format!("{host_float}");  // Rust's f64 Display: shortest
                                      // round-trip decimal, e.g.
                                      // "123.456", not "123.4560000001"
Rational::from_decimal_literal(&text)
```

This reuses the *same* code path an ordinary `.my` program already
goes through for a typed decimal literal — a host-supplied
`player-position/read` float becomes indistinguishable, once inside
the language, from a program that had simply written that number as a
literal. No new numeric machinery is needed; the boundary conversion
is "format the float as its shortest decimal string, then parse it
exactly," not a new algorithm.

## Why this beats the raw-binary-value alternative

Converting via the *exact binary value* (e.g. `183369104568320000/
1486058960531456` instead of `123.456`) would be a strictly more
"faithful" conversion in one sense — no information about the exact
bit pattern is discarded — but every practical `.my` program reading a
player position wants "123.456," not the binary artifact of how IEEE754
happens to represent it. The shortest-round-trip-decimal path is what
`from_decimal_literal` already does for every literal in every `.my`
file in this ecosystem; treating a host float differently would be
inventing a second, inconsistent numeric on-ramp for no real benefit —
the game engine's own float already lost whatever "true" real-number
precision existed at the physics-simulation level, so preserving its
exact bit pattern doesn't preserve anything meaningful beyond what the
shortest decimal already captures.

## What this means concretely for `player-position/read`

The capability returns a list/tuple of `Value::Rational` (or
`Value::Number(_, Exactness::Exact)` when `as_precise_i64` happens to
apply, matching the existing `exact_value` compression convention in
`crates/my-lisp/src/eval/arithmetic.rs`), never `Exactness::Inexact`.
The adapter-side conversion is exactly the `format!` + parse path
above, done once per coordinate, at the same boundary point the
capability already marshals RTTI data into `my-lisp` `Value`s — no
change to my-lisp's own core is needed, this is purely an adapter
(wsm-my-lisp) responsibility using an already-public my-lisp API.

`lib/rational.my`'s exact-rational math (rat-sqrt, rat-power, etc.,
found untracked in this repo earlier this session) becomes directly
relevant here rather than orphaned: once a game position is a
`Rational`, computing distances (`rat-sqrt` of a sum of squares) with
the same exactness guarantee follows naturally, without a silent
precision-losing round-trip through `f64` anywhere in the pipeline —
exactly the esp32-pendulum precedent.
