# Foreign C ABI — design (fixed before code)

Owner research 2026-09-12: a study of Guile's C interop shows two
genuinely different problems that this repository's polyglot work has
so far treated as one. `docs/POLYGLOT-SEMANTIC-ORCHESTRATOR-IMPLEMENTATION-PLAN.md`
solves "talk to Python" — a separate runtime, reached over a
subprocess/TCP transport. Native C libraries (BLAS, FFTW, libc, CUDA
drivers) are a different problem: there is no separate runtime to
connect to, only a shared library already loadable in-process. A
subprocess would only add overhead and a serialization boundary where
none is needed. This document is the fixed-before-code design for that
second problem, following this repository's own design-before-code
discipline (`docs/cml-semantic-export-v1-design.md`,
`docs/FOREIGN-RUNTIME-PROTOCOL-DESIGN-2026-09-11.md`). **No
implementation lands with this document.**

## What Guile actually proved, distilled

Guile does not have `call-sin`, `call-memcpy`, `call-fftw`. It has
exactly one generic mechanism, and everything else is Scheme code built
on it:

```text
foreign library handle
        +
symbol address (a function's address inside that library)
        +
ABI signature (return type, argument types)
        │
        ▼
pointer->procedure
        │
        ▼
an ordinary Scheme procedure
```

`(sqrt 2.0)` calling into `libm` needs no C wrapper written for `sqrt`
specifically — only the generic three-part mechanism above, applied
once with `libm`, `"sqrt"`, and `(double) -> double` as the arguments.
Guile also proves the mirror direction is the same shape: a Scheme
procedure becomes a real C function pointer via `procedure->pointer`
(for callbacks into C, e.g. `qsort`'s comparator), and Guile wraps a
raw `void*` as a distinct foreign-pointer value — a mechanical resource
with an optional GC finalizer, never promoted to language semantic
identity.

The lesson for my-lisp is not "copy Guile." It is: **one generic ABI
substrate, described by data (library, symbol, signature), replaces an
unbounded set of per-library Rust wrappers.** This is the same
reuse-first, no-per-capability-Rust-code principle already applied to
the Python track (`lib/foreign.my` over existing TCP) and to the
FS-CAPABILITY-UTF8-POLICY-MIGRATION slice — applied here to native
code instead of a subprocess.

## Why this is a separate track from the Python/subprocess design

| | Python track (existing) | C ABI track (this document) |
|---|---|---|
| Target | a separate runtime process | code already loadable in-process |
| Transport | TCP, canonical S-expression envelope | none — direct in-process call |
| Value crossing | copy through a wire format | direct memory / ABI calling convention |
| Failure isolation | a crashed Python process doesn't crash my-lisp | a bad C call CAN crash the host process — no sandboxing is possible at this layer |
| Identity | `(foreign-ref N)`, bridge-table bookkeeping | a foreign pointer/library handle, mechanical resource |

The two share the *shape* of the lesson (generic substrate + data
describing what to call, not one Rust function per foreign name) but
cannot share a transport, because there is no transport in the C case
— reusing the Python design's TCP/subprocess machinery here would be
exactly the kind of premature abstraction this repository's own
discipline (`docs/agent-doctrine.md` rule 7) warns against.

## Target architecture

```text
                    MY-LISP
                       │
                  lib/ffi.my
      (foreign-library-open/symbol/call, ABI type descriptors)
                       │
       ┌───────────────┼───────────────┐
       ▼               ▼               ▼
  library-open-raw  symbol-address-raw  ffi-call-raw
       │               │               │
       └───────────────┼───────────────┘
                        ▼
                 minimal Rust substrate
             (OS loader + libffi-style call)
                        │
                        ▼
              .so / .dll / .dylib
              (BLAS, FFTW, libc, CUDA, ...)
```

## What my-lisp owns vs. what stays mechanism

**Rust owns** (irreducible — none of this is expressible in Lisp
without a foreign-function-call primitive, unlike the Python case
where the subprocess/TCP substrate already existed):
- Opening a shared library by path (`dlopen`/`LoadLibrary` equivalent).
- Resolving a symbol's address inside an open library.
- Actually invoking a C function pointer with a given calling
  convention, argument types, and return type — this needs either a
  hand-written per-arity dispatch table or a real ABI-call library
  (e.g. `libffi`), because Rust cannot construct an arbitrary C call at
  runtime without one.
- Reading/writing raw memory through a pointer, for the small set of
  primitive scalar types (matching Guile's own type vocabulary:
  `int8`/`uint8`/.../`int64`/`uint64`/`float`/`double`/pointer).

**Lisp owns** (the actual semantic surface, per `lib/ffi.my`):
- `foreign-library-open`, `foreign-symbol`, `foreign-signature`,
  `foreign-call`, `foreign-pointer?`, `foreign-library-close`.
- Every specific binding: `(define sqrt (foreign-call libm "sqrt" '(double) 'double))`,
  `(define memcpy ...)`, `(define blas-dgemm ...)` all live in ordinary
  `.my` files, never in Rust. Zero Rust code is added when a new C
  function needs binding — that is the whole point.
- ABI type descriptors as ordinary Lisp data (symbols like `int32`,
  `double`, `pointer`), not a closed Rust enum grown per type — though
  the *set* of primitive scalar types the raw substrate understands is
  necessarily fixed in Rust (mirroring Guile's own fixed
  `int8`..`double`/`*` vocabulary; this is mechanism, not semantics).

## Value representation: no `Value::Foreign` here either

Same reasoning as the Python track's own reuse-first correction: a
foreign pointer or library handle is a **mechanical resource**, not a
new semantic identity. Two live options, to be decided in a Phase 0
comparison before any code (mirroring the Python track's own Decision
A):

- **Model 1**: reuse the existing opaque-capability-handle pattern
  already used for `TcpConnection` (a first-class `Value` variant that
  is nothing but an opaque resource handle with `eq`-by-handle
  identity) — extended to a `ForeignPointer`/`ForeignLibrary` shape.
- **Model 2**: represent a foreign pointer as plain serializable Lisp
  data (an integer address plus a library tag), exactly like the
  Python track's `(foreign-ref N)` — with the raw substrate itself
  refusing to dereference an address it did not itself hand out, so an
  ordinary Lisp integer can never be used to forge a pointer.

Model 1 mirrors Guile's own choice (a distinct foreign-pointer object,
not a bare integer) and gets GC-finalizer-style cleanup for free from
the existing `Rc`-based value lifetime discipline already used for
`TcpConnection`; Model 2 needs no new `Value` variant at all but pushes
forgery-prevention into the raw substrate. Which one is chosen is
**not decided by this document** — it is the first real implementation
task, done the same way the Python track's Decision A was: a real
comparison, not a default.

## Guile's second mechanism (`SCM`/`scm_c_define_gsubr`) is explicitly not needed

Guile's C-embeds-Scheme direction (`libguile`, `SCM`, `scm_call_n`,
`scm_boot_guile`) exists because Guile is designed to be *embedded
into* C applications that don't otherwise know about Scheme. my-lisp's
own embedding story already runs the other way — `my-lisp-host`/
`my-lisp-cli` embed the language, and `crates/my-lisp`'s own public
Rust API (`Session`, `eval_program`, `Value`) already plays exactly
this role for Rust embedders. There is no missing capability here that
`SCM`-style API would add; it is not part of this design.

## Callbacks (`procedure->pointer`) are out of scope for v0

A C library calling back into a Lisp closure (Guile's `qsort`-comparator
example) needs a Lisp closure to be materialized as a real C function
pointer at a fixed address with a fixed calling convention — a
meaningfully harder mechanism than the one-directional call this
document scopes (it needs a trampoline generated per signature, or a
fixed small set of pre-generated trampolines). Real, valuable, and
explicitly **not attempted in v0** — a one-directional `foreign-call`
(Lisp calls C) is the entire v0 surface. Flagged the same way the
Python track flagged NumericBuffer/NumPy as a separate research track:
important, but conflating it with the base mechanism risks never
shipping either.

## NumericBuffer ↔ native memory (the "may matter more than Python" claim)

The owner's strongest claim in this research is that `NumericBuffer`
(`i32-buffer`/`f32-buffer`, already existing) reaching BLAS/FFTW
directly through a pointer view, with **zero copy**, may be more
valuable than the Python bridge, because C ABI has no subprocess
boundary forcing a copy the way Python's does. This is real and worth
pursuing, but it is a DATA-plane question layered on top of the
CONTROL-plane mechanism above (`foreign-library-open`/`foreign-call`),
exactly mirroring the Python track's own control/data-plane split. It
is not designed further here — it becomes real only after the base
one-directional call mechanism (library open, symbol resolve, scalar
in/out call) is proven with a real library, per this repository's own
vertical-slice discipline.

## Explicit non-goals for v0

- `procedure->pointer`-style callbacks (C calling back into Lisp).
- Struct-by-value marshaling (Guile's own foreign-structs layer) —
  scalar arguments and pointers only for v0.
- NumericBuffer/native-memory zero-copy bridge — separate research
  track, not mixed with the base call mechanism (mirrors the Python
  track's NumPy/NumericBuffer non-goal).
- Any specific library binding (`lib/blas.my`, `lib/fftw.my`) beyond
  one proof-of-concept call (e.g. `libm`'s `sqrt`) needed to validate
  the mechanism.
- Any new `Value` variant chosen without the Model 1 vs. Model 2
  comparison actually being written down first.

## What accompanies this document

Nothing. No Rust code, no `lib/ffi.my`, no `Value` change. Per the
owner's own instruction pattern for the Python track
(`docs/POLYGLOT-SEMANTIC-ORCHESTRATOR-IMPLEMENTATION-PLAN.md`'s "plan
first, task DAG in tasks.my, do not start POLYGLOT-002+ in the same
commit"), this pass adds only this design document and the
corresponding `tasks.my` entries.
