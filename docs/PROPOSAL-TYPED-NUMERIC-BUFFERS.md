# PROPOSAL: immutable typed numeric buffers

**Status:** PROPOSED M0, not implemented and not part of language contract 2.1  
**Date:** 2026-08-24  
**Type:** additive language/value proposal; ratification would require contract 2.2  
**Driver:** portable CPU/GPU/FPGA bulk execution through CML without weakening exact arithmetic

## 1. Problem

my-lisp already has a general `Vector` value with O(1) indexing. It is not a
GPU buffer contract:

- it is heterogeneous (`Vec<Value>`);
- it is mutable through `vector-set!`;
- its elements may be pairs, closures, bignums, rationals, or host handles;
- CML has no corresponding typed contiguous representation;
- silently converting exact values to hardware floats would violate S1.

GPU hardware needs contiguous storage, a fixed element width, uniform
operations, and an explicit numeric domain. These facts must become visible
without redefining ordinary lists, vectors, integers, or rationals.

## 2. Proposed value domain

Add a distinct immutable value family:

```text
NumericBuffer
  +-- I32(Arc<[i32]>)
  `-- F32(Arc<[f32]>)
```

M0 deliberately has only `i32` and `f32`:

- `i32` gives a fixed-width integer domain whose values convert exactly to
  and from my-lisp exact integers while in range;
- `f32` makes approximation explicit and maps portably to the minimum useful
  compute-shader numeric domain;
- `i64`, `f64`, unsigned types, shapes, and multidimensional tensors require
  backend capability evidence and earn later additive revisions.

This does not replace `Vector`. A general vector remains the correct value for
heterogeneous or mutable indexed data; a numeric buffer is the correct value
for immutable bulk computation.

## 3. Proposed surface

Constructors and accessors:

```lisp
(i32-buffer 1 -2 3)
(f32-buffer 1 1/10 -2.5)
(numeric-buffer? value)
(numeric-buffer-type value)       ; => i32 or f32
(numeric-buffer-length value)
(numeric-buffer-ref value index)  ; => exact integer or inexact number
```

Canonical reader/printer forms, required before ratification:

```lisp
#i32(1 -2 3)
#f32(1.0 0.10000000149011612 -2.5)
```

`read(write-to-string(buffer))` must reconstruct the same element type and
bits. Constructor syntax is convenient source code; the tagged reader form is
the canonical serialized value. The exact spelling remains proposed until the
reader implementation and round-trip fixtures exist.

No `numeric-buffer-set!` exists. Transformations return a new buffer. A host
backend may reuse physical allocation only when that optimization is
unobservable.

## 4. Conversion rules

### i32

- Accept exact integer values in `[-2147483648, 2147483647]`.
- Reject rationals with a denominator other than one.
- Reject inexact values even if their displayed magnitude is integral.
- Out of range is `NumericOverflow`; wrong numeric kind is `Type`.
- Reading an element returns an ordinary exact my-lisp integer.

There is no truncation, saturation, or wrapping during construction.

### f32

- The constructor is an explicit request to enter the inexact binary32
  domain; this is not an implicit compiler conversion.
- Accept finite exact or inexact numeric inputs representable as finite
  IEEE-754 binary32 after round-to-nearest, ties-to-even.
- Reject overflow to infinity as `NumericOverflow`.
- NaN and infinities are absent from M0 source semantics.
- Reading an element returns an ordinary inexact my-lisp number carrying the
  exact value of the stored binary32 bit pattern promoted for the host value.

Consequently `(f32-buffer 1/10)` visibly stores the binary32 approximation;
the canonical printer exposes the round-trippable value. No result may be
labelled exact after this conversion.

## 5. Arithmetic and overflow

Typed buffers establish representation, not a second set of scalar operators.
Bulk operations are introduced through CML Compute IR and must define their
domain explicitly.

- `f32` operations follow the declared binary32 operation and comparison
  contract; differential tests use bit equality where deterministic and a
  named tolerance only where the operation permits backend variation.
- `i32` must not silently inherit GPU wrapping arithmetic. M0 offload is
  allowed only when a range proof excludes overflow or when a kernel returns a
  checked overflow flag. Failure is `NumericOverflow`, matching the language's
  no-silent-corruption rule.

Backend selection may change performance, never the observable value or error.

## 6. Effects, aliasing, and concurrency

Numeric buffers are immutable values with structural equality:

```text
same element type + same length + same element bits => equal
```

`i32` equality compares integer elements. `f32` equality compares the stored
binary32 values under the eventual inexact-number identity rule; the
reader/printer must preserve bits needed by that rule.

Immutability gives CML the aliasing fact required for parallel execution:
concurrent readers cannot observe partial writes. Device upload, kernel
execution, and download are runtime implementation details. A result becomes
visible only after successful completion; device loss falls back before
execution or returns a named backend error according to the planner contract,
never a partial buffer.

## 7. CML boundary

CML may lower a ratified buffer into semantic IR metadata:

```text
Buffer {
  element_type: I32 | F32,
  length,
  storage: Contiguous,
  mutability: Immutable
}
```

GPU eligibility still requires all of:

```text
bulk shape (element-wise or reduction)
+ pure kernel
+ contiguous typed buffer
+ backend supports the element type and operation
+ semantic overflow/rounding obligations are satisfied
```

Unknown facts reject offload. CPU is the mandatory reference backend. Source
code does not name CUDA, Vulkan, WGSL, or a GPU vendor.

## 8. FPGA boundary

This proposal does not allocate a fpga-lisp tag, opcode, or BRAM bank. After
the language representation is ratified, fpga-lisp may propose a descriptor
and data-bank ABI independently. General Lisp execution may keep buffers as
unsupported values while a specialized dataflow backend consumes static
buffers. No RTL should precede the joint representation and ABI fixtures.

## 9. Contract decision

Ratification is an additive language change and therefore a proposed 2.1 ->
2.2 minor bump. The bump happens only after these acceptance gates pass:

1. reader and printer round-trip both element types;
2. construction and indexing errors have named kinds;
3. existing `Vector` behavior remains unchanged;
4. structural equality is specified and tested;
5. CML CPU backend matches the my-lisp oracle;
6. exact-to-f32 conversion is explicit in every source path;
7. WASM either implements the same value or returns an explicit unsupported
   capability without changing native semantics.

Until then this document is a proposal. CML's `compute-contract.my` 0.1 may
model a hypothetical proven contiguous representation for analysis tests, but
must not claim that my-lisp currently supplies one.

## 10. Rejected shortcuts

| Shortcut | Reason rejected |
|---|---|
| Treat current `Vector` as a GPU buffer | heterogeneous, mutable, no element-width contract |
| Convert lists to f32 automatically | changes exact values and hides transfer/allocation |
| Put CUDA calls in Lisp source | vendor API becomes language semantics |
| Make every numeric operation GPU eligible | irregular control and small inputs lose; effects may be unsafe |
| Use unchecked i32 wrapping | disagrees with named-overflow/no-silent-corruption semantics |
| Add FPGA tags before fixtures | makes physical representation dictate language semantics |

## 11. Implementation slices

1. Failing my-lisp value/reader/conversion fixtures.
2. `NumericBuffer` value representation and canonical rendering.
3. Constructors/accessors and WASM parity.
4. CML semantic Buffer IR and lowering.
5. CPU ComputeBackend reference implementation.
6. Portable Rust GPU backend for one element-wise f32 kernel.
7. Differential CPU/GPU/oracle evidence and measured planner threshold.

