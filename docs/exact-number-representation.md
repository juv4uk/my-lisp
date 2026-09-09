# Exact-number representation audit

Status: **audited on 2026-09-09 for issue #28**.

This document describes representation, not a second numeric semantics. The semantic claim is deliberately narrower and stronger than “we do not use floats”:

> **Exact arithmetic never silently rounds.** The runtime may use `f64` as a compact representation only for mathematically exact integers in the IEEE-754 binary64 exact-integer range; larger or fractional exact values use arbitrary-precision `Rational`.

## Runtime representations

| Representation | Meaning | Admission rule | Observable consequence |
|---|---|---|---|
| `Value::Number(f64, Exactness::Exact)` | exact integer | denominator is 1 and integer is within `[-2^53, +2^53]` | may participate in exact arithmetic; exactness tag is semantic data |
| `Value::Rational(Rational)` | arbitrary-precision exact integer/fraction | exact value cannot be losslessly compressed to the compact representation, or is fractional | no numeric ceiling except configured/resource limits and available memory |
| `Value::Number(f64, Exactness::Inexact)` | deliberately inexact binary64 value | explicit inexact ingress or arithmetic involving an inexact operand | arithmetic/comparison follows inexact binary64 semantics |
| `NumericBuffer::I32` | explicit fixed-width integer data | exact integer must fit signed 32 bits | narrowing failure is named `NumericOverflow` |
| `NumericBuffer::F32` | explicit binary32 data | finite numeric input is intentionally narrowed | precision loss is part of the buffer conversion, not relabelled as exact scalar arithmetic |

`Rational::as_precise_i64()` is the single compactness predicate for ordinary exact scalar values. Despite the method name, it is stricter than “fits i64”: it returns `Some` only for an integer within the binary64 exact-integer range ±2^53.

## Transition map

```text
source integer / decimal / exponent / n/d
                  |
                  v
          exact Rational parse
                  |
          +-------+--------+
          |                |
 integer, |n| <= 2^53      otherwise
          |                |
          v                v
 Number(f64, Exact)     Rational
          \                /
           \              /
            v            v
           exact arithmetic
              as Rational
                  |
        +---------+----------+
        |                    |
 result compactable      result not compactable
        |                    |
        v                    v
 Number(f64, Exact)       Rational
```

### Parser

`crates/my-lisp/src/parser.rs` first constructs exact numeric meaning. Integer, decimal and exponent spellings are never parsed through an intermediate binary floating-point approximation. Decimal/scientific text goes through `Rational::from_decimal_literal`; large integer text goes through arbitrary-precision `Rational::from_literal`.

After exact parsing, `as_precise_i64()` decides whether an integer may be stored compactly. Therefore:

```text
9007199254740992   -> compact exact Number   (+2^53)
9007199254740993   -> Rational               (+2^53 + 1)
-9007199254740992  -> compact exact Number   (-2^53)
-9007199254740993  -> Rational               (-2^53 - 1)
0.1                -> Rational 1/10
1,25               -> Rational 5/4
```

A syntactically valid decimal/exponent literal that exceeds the reader resource cap fails as `NumericOverflow`; it must not fall back to a symbol.

### Evaluator and arithmetic

`crates/my-lisp/src/eval/arithmetic.rs` normalizes every exact scalar operand to `Rational` before the general exact path. A compact exact `Number` is reconstructed as an exact integer Rational; a `Value::Rational` is already exact.

All-exact `+`, `-`, `*`, `/` use Rational operations. A result is compressed only through `exact_value()`, which delegates to the ±2^53 predicate above.

There is an i64 fast path for common exact integer `+`, `-`, `*`. The #28 audit found a real invariant violation there: after a successful checked-i64 operation the old code returned `Value::Number(result as f64, Exactness::Exact)` directly. `checked_*` proves i64 range, **not** binary64 exactness. For example:

```text
(* 3000000001 3000000001)
= 9000000006000000001
```

Both operands qualify for the fast path, the product fits i64, but the odd result is above 2^53 and cannot be represented exactly as f64. The audit fixes this by routing the fast-path result through `exact_value()` as well. `crates/my-lisp/tests/exact_number_boundaries.rs` is the falsification test that would fail if this silent-rounding path returned.

If any arithmetic operand is deliberately inexact, arithmetic converts the participating numeric values to f64 and returns `Exactness::Inexact`. This is an explicit semantic transition, not an exact-value compression.

### Comparison and identity

Magnitude comparisons `<`, `=`, `>` compare all-exact inputs as Rational values. If an inexact operand participates, the comparison uses inexact f64 magnitudes.

`eq` remains value identity/equality and includes representation/exactness distinctions defined by `Value::PartialEq`; it is not a substitute for numeric magnitude `=`. Do not collapse these two relations merely to simplify numeric representation.

### Printing and read-back

- compact exact Number prints as its integer;
- an integer Rational prints its numerator without `/1`;
- fractional Rational prints `numerator/denominator`;
- an integral inexact Number prints with a decimal marker such as `3.0`, so read-back does not silently look exact.

The boundary corpus proves that a large exact integer can pass `write-to-string -> read -> eval` without magnitude loss.

### FASL

`crates/my-lisp/src/syntax.rs` gives `ExprKind::Number` and `ExprKind::Rational` distinct FASL tags. Number snapshots store all f64 bits plus the `Exactness` tag; Rational snapshots store arbitrary-precision numerator and denominator limbs. The existing `fasl_round_trip_is_byte_identical_and_hash_bound` test structurally exercises both Number and Rational source forms and requires byte-identical re-encoding after decode.

FASL therefore preserves the parser's numeric representation; it does not re-decide exactness through a float conversion.

### Typed numeric buffers

Typed buffers are an explicit narrowing boundary, not the scalar exact arithmetic model:

- `#i32(...)` accepts exact integers and fails `NumericOverflow` outside signed 32-bit range.
- `#f32(...)` intentionally narrows finite numeric values to IEEE-754 binary32. An exact `1/10` becoming the nearest f32 is allowed because the requested target representation is explicitly `f32`; it must never re-enter scalar arithmetic labelled `Exact` without an explicit semantic conversion.

`crates/my-lisp/tests/exact_number_boundaries.rs` records both cases.

## Known inexact ingress

Current code search finds ordinary `Value::Number(..., Exactness::Inexact)` creation in narrowly identified places:

1. arithmetic whose inputs already include an inexact value;
2. JSON numeric parsing, where the external JSON number is interpreted through f64;
3. f32-buffer higher-order mapping bridges, where buffer elements are deliberately surfaced to the callback as inexact f64 values.

Source-language decimal literals themselves are exact; `0.1` is `1/10`, not a binary64 approximation.

## Executable evidence

Primary #28 regression/boundary corpus: `crates/my-lisp/tests/exact_number_boundaries.rs`.

It proves:

- exact compact boundary at ±2^53;
- one-past-boundary values stay Rational;
- fast-path multiplication cannot relabel a rounded f64 as exact;
- arithmetic crossing Number/Rational representation boundaries preserves magnitude;
- decimal/comma/exponent literals stay exact;
- exact division preserves reduced fractional and large-integer values;
- write/read/eval round-trip preserves a large exact integer;
- parser resource limits remain named `NumericOverflow`;
- i32/f32 buffers remain explicit narrowing boundaries.

Existing supporting evidence includes parser decimal/resource-limit tests, arbitrary-precision arithmetic tests in `mccarthy.rs`, FASL structural round-trip tests in `syntax.rs`, and typed-buffer coverage in `numeric_buffers.rs` / `decimal_comma.rs`.

## Allowed claim vocabulary

Preferred:

> **Exact arithmetic never silently rounds. The runtime may use `f64` as a compact representation only for mathematically exact integers in its exact range; larger or fractional exact values use arbitrary-precision `Rational`.**

Also accurate:

- “my-lisp has arbitrary-precision exact integers and rationals”;
- “source decimal literals are exact”;
- “inexact arithmetic is explicit once an inexact value enters the computation.”

Avoid unless the implementation changes:

- “all numbers are Rational”;
- “my-lisp does not use floats”;
- “every numeric boundary is exact” (typed f32 buffers and external JSON numbers are deliberately inexact boundaries).

## Stop condition

Any future path that constructs `Value::Number(_, Exactness::Exact)` from an integer without proving it lies in the binary64 exact-integer range is a semantic regression, even if its Rust integer calculation did not overflow.
