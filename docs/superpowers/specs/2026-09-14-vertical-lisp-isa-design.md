# Vertical Lisp ISA Architecture

**Date:** 2026-09-14
**Status:** proposed / approved in conversation; implementation not started
**Scope:** `my-lisp` machine boundary, Intel Core i5-6400 (Skylake), x86-64

## Goal

Move `my-lisp` from a runtime in which Rust owns both mechanism and much of execution behavior toward a vertical architecture in which:

- `my-lisp` remains the only authority for language meaning and semantic IDs;
- the processor ISA is represented inside `my-lisp` as machine facts, not as language semantics;
- instruction encoding is progressively owned by Lisp;
- semantic lowering selects from available ISA capabilities;
- Rust is reduced to a bootstrap/mechanism layer rather than a semantic authority;
- CML remains an optimizer/compiler consumer of the same machine facts, but is not the exclusive owner of machine instruction identity.

This is an incremental migration. Rust is not removed first; it is bypassed where Lisp can already own the next lower layer correctly.

## Non-goals

This design does **not**:

- turn x86 instructions into automatic public Lisp semantic functions;
- allocate semantic IDs from machine instructions;
- create a new Lisp CPU or FPGA target;
- require the whole x86-64 ISA to be executable from Lisp in the first slice;
- remove the operating system from memory protection, process, file, network, or device responsibilities;
- replace CML as an optimizing compiler.

## Authority model

There are three different authorities and they must remain separate.

### 1. Semantic authority

`lib/surface/semantic-registry.lisp` owns language meaning and stable semantic identities.

Example:

```text
0104 = semantic identity for `додати`
```

No machine fact may allocate or redefine a semantic identity.

### 2. ISA authority

The physical instruction set is an external hardware fact.

For Intel x86-64 the normative input is derived from Intel documentation and Intel XED. The repository may store a normalized Lisp representation of those facts, but the existence and encoding of `ADD`, `PDEP`, `AESENC`, `VPMADDUBSW`, etc. do not originate from the language contract.

### 3. Optimization authority

CML may choose schedules, instruction sequences, register allocation, vectorization, target-specific optimizations, and alternative lowering strategies. It does not own the existence or identity of the processor's instructions.

## New vertical boundary

The old direction was effectively:

```text
my-lisp semantics
    ↓
Rust evaluator / runtime
    ↓
compiler/backend machine knowledge
    ↓
machine bytes
    ↓
CPU
```

The target direction is:

```text
                    MY-LISP
                       │
                semantic registry
                       │
                semantic lowering
                       │
                       ▼
                 ISA catalogue
                       │
              Lisp-owned encoder
                       │
                       ▼
                 machine bytes
                       │
          host executable-memory mechanism
                       │
                       ▼
                 physical CPU
```

Rust may remain below the language as a bootstrap mechanism for memory allocation, page protection, and native invocation. It must not need to know that semantic ID `0104` means addition, nor choose `ADD` on behalf of Lisp in the proof path.

## Repository structure

Target layout:

```text
lib/machine/
├── isa/
│   ├── x86-base.lisp
│   ├── x86-64.lisp
│   ├── x87.lisp
│   ├── mmx.lisp
│   ├── sse.lisp
│   ├── sse2.lisp
│   ├── sse3.lisp
│   ├── ssse3.lisp
│   ├── sse4.1.lisp
│   ├── sse4.2.lisp
│   ├── aes.lisp
│   ├── pclmulqdq.lisp
│   ├── avx.lisp
│   ├── f16c.lisp
│   ├── fma.lisp
│   ├── bmi1.lisp
│   ├── bmi2.lisp
│   ├── avx2.lisp
│   ├── rdrand.lisp
│   ├── rdseed.lisp
│   ├── adx.lisp
│   ├── xsave.lisp
│   └── ...
│
├── cpu/
│   └── intel-core-i5-6400.lisp
│
├── encoding/
│   └── x86-64.lisp
│
└── lowering/
    └── semantic-x86-64.lisp
```

The ISA catalogue is grouped by extension/standard, not by current Lisp functions.

## ISA catalogue schema

The machine catalogue must be useful even for instructions with no Lisp semantic mapping.

Conceptual row:

```lisp
(instruction ADD
  (extension X86-BASE)
  (class integer-arithmetic)
  (forms
    (... encoding metadata ...))
  (privilege user)
  (availability supported))
```

Required dimensions for each instruction or instruction form:

- mnemonic / instruction identity;
- ISA extension or standard;
- operand form;
- opcode bytes / opcode map;
- legacy prefixes when relevant;
- REX/VEX metadata when relevant;
- ModR/M and SIB requirements;
- immediate/displacement width;
- 64-bit mode validity;
- privilege class;
- feature requirement;
- support status for the target CPU profile.

The catalogue may initially cover a proof subset, but its schema must scale to the complete target ISA.

## CPU profile

`lib/machine/cpu/intel-core-i5-6400.lisp` describes the concrete target as the intersection of:

```text
Intel/XED Skylake capability sets
        ∩
Core i5-6400 SKU capabilities
        ∩
runtime CPUID/XGETBV availability
        ∩
user-mode legality
```

A generic Skylake feature is not automatically assumed to be usable on this SKU.

The profile groups supported capabilities by standards/extensions such as BASE, SSE2, AVX, BMI2, AVX2, AES, and others actually available on the machine.

## Lisp-owned encoder

The encoder consumes an instruction form and operands and produces machine bytes.

It must be ordinary Lisp code and ordinary Lisp data wherever practical.

Example proof target:

```text
ISA catalogue: ADD r64,r64 form exists
        ↓
Lisp encoder
        ↓
48 01 D8
```

The first encoder slice needs only enough x86-64 machinery for a small proof subset:

- REX prefix;
- opcode emission;
- register encoding;
- ModR/M;
- immediate encoding where needed;
- `RET`;
- a minimal set such as `MOV`, `ADD`, `SUB`, `CMP`, conditional branch or return.

It must not depend on a textual external assembler for this proof.

## Host/bootstrap boundary

The host is allowed to own physical mechanisms that the OS controls:

- allocate writable memory;
- write bytes;
- transition pages from writable to executable where required;
- invoke a code address using a defined ABI;
- release memory.

Prefer W^X discipline: writable then executable, not permanently RWX.

The host must not contain semantic knowledge of the Lisp operation being tested.

## Semantic lowering

Semantic lowering maps an existing semantic identity to machine operations only after the ISA layer is independently available.

Example:

```text
0104 / додати
      ↓
representation check
      ↓
small exact integer path
      ↓
ADD from ISA catalogue
      ↓
Lisp encoder
      ↓
bytes
```

For arbitrary-precision integers or rationals, `0104` keeps its existing exact semantics and uses a wider Lisp/runtime routine. `ADD` is therefore a physical realization of one representation path, not the meaning of `0104`.

## Relationship to CML

CML remains valuable and is not bypassed as an optimizer.

The intended relationship is:

```text
              shared machine facts
                     │
          ┌──────────┴──────────┐
          ▼                     ▼
 my-lisp direct lowering       CML
 proof / bootstrap path      optimizer
          │                     │
          └──────────┬──────────┘
                     ▼
                  target CPU
```

CML may ultimately generate better code. The direct Lisp path exists to prove that machine access is not semantically owned by CML or Rust.

## Relationship to PR #118

The current `lib/machine/intel-core-i5-6400.lisp` in PR #118 is a semantic-to-machine projection for 62 semantic IDs. It is useful evidence but is not the final machine architecture.

It should evolve into:

1. a complete, standard-grouped ISA catalogue independent of semantic functions;
2. a CPU capability profile;
3. a small Lisp-owned encoder;
4. a separate semantic-lowering table or Lisp functions that reference the catalogue.

The human function table may continue to display machine realizations, but it must consume them as projections and never become an ISA authority.

## Change to `machine-lowering-boundary.lisp`

The existing boundary says `compiler-authority cml` and `machine-instruction-identity compiler-owned`. That is incompatible with this design and must be replaced deliberately rather than worked around.

Target authority statements:

```text
semantic-authority          my-lisp
isa-authority               hardware-specification
isa-source                  intel-xed/intel-sdm
isa-representation          my-lisp
instruction-encoding        my-lisp
optimization-authority      cml
semantic-id-from-isa        forbidden
raw-execution-mechanism     host
```

The one-way semantic rule remains:

```text
machine fact → semantic ID     forbidden
semantic meaning → machine use allowed
```

## Incremental proof ladder

### Stage 0 — catalogue proof

Represent a small Intel 64 subset as Lisp data grouped by ISA extension and verify the entries against known encodings.

Success means Lisp can query the catalogue without Rust semantic tables.

### Stage 1 — encoding proof

Lisp encodes one or more instructions to exact expected byte sequences.

Example:

```text
ADD rax,rbx → 48 01 D8
RET         → C3
```

Success means the bytes are selected and constructed by Lisp code.

### Stage 2 — native execution proof

Lisp constructs a tiny no-argument native function that returns a known value. The host only provides executable-memory and call mechanisms.

Success means:

```text
Lisp data → Lisp encoder → bytes → i5-6400 → expected result
```

### Stage 3 — semantic proof

An existing semantic operation uses the ISA catalogue and Lisp encoder.

Candidate first proof:

```text
(+ 2 3)
```

with dual execution:

```text
interpreter path → 5
native Lisp-owned lowering → 5
```

Success requires that Rust does not know that `0104` means addition in the native proof path.

### Stage 4 — Lisp data-model proof

Lower a Lisp structural operation such as `car`/`cdr`, followed eventually by `cons` after representation/allocation rules are explicit.

This is a stronger proof than arithmetic because it demonstrates the Lisp data model itself on the physical machine.

### Stage 5 — coverage growth

Expand the ISA catalogue by standards/extensions and let semantics consume machine capabilities as useful. The language does not need one public Lisp function per CPU instruction.

## Testing strategy

Tests are layered so claims remain precise.

### Catalogue tests

- no duplicate instruction-form identity;
- every form belongs to a declared extension;
- CPU profile references only declared extensions/forms;
- unsupported/privileged instructions are not advertised as ordinary user-mode executable capabilities.

### Encoder tests

Golden byte tests for each implemented form.

For every encoded proof instruction:

```text
Lisp instruction form → exact expected bytes
```

### Boundary tests

- no ISA entry may allocate a semantic ID;
- semantic registry contains no raw target instruction namespace introduced by the machine catalogue;
- host primitive names remain mechanism-oriented;
- the native proof cannot call an external assembler.

### Native witness

Execute generated code on x86-64 CI where supported and verify the result. Non-x86 CI may validate catalogue/encoding only.

### Semantic parity

For each semantic function admitted to direct lowering, compare interpreter/reference result with native result over a bounded test corpus.

## Error handling and portability

Machine operations must distinguish at least:

- instruction not in catalogue;
- form not encodable for operands;
- CPU feature unavailable;
- OS state unavailable (for example AVX state not enabled);
- privileged instruction forbidden in user mode;
- executable memory unavailable;
- semantic representation outside a direct machine fast path.

A failed fast path must fall back to correct Lisp semantics when a valid fallback exists; it must never silently change meaning.

## Scientific claim discipline

Do not call the project a fully vertical Lisp merely because the catalogue exists.

Claims should track evidence:

1. **ISA represented in Lisp** — after catalogue proof.
2. **Lisp-owned machine encoding** — after golden encoding tests.
3. **Lisp emits and executes native code** — after Stage 2.
4. **Lisp semantics directly reaches CPU through Lisp-owned lowering** — after Stage 3.
5. **Vertical Lisp architecture** — only after multiple core semantic operations, including at least one structural Lisp operation, survive interpreter/native parity tests.

The name of the achievement must never be stronger than the strongest experiment supporting it.

## First implementation slice

The first retained implementation after this design is approved should be intentionally small:

1. replace the old compiler-owned machine-boundary assertions with the new three-authority model;
2. introduce `lib/machine/isa/x86-base.lisp` and `lib/machine/cpu/intel-core-i5-6400.lisp` in the new schema;
3. move a tiny proof set (`MOV`, `ADD`, `RET`) into the ISA catalogue;
4. implement their encoding in Lisp;
5. add golden byte tests;
6. only then add the host native-execution mechanism and the first execution witness;
7. only after that connect semantic ID `0104` or another suitable semantic identity.

No attempt should be made to populate the entire Intel ISA before the schema and proof path are demonstrated end to end.
