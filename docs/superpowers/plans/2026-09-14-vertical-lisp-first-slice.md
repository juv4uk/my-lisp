# Vertical Lisp First Slice Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Prove the first retained vertical path in which my-lisp represents x86-64 ISA facts, encodes `MOV`/`ADD`/`RET` in Lisp, and executes Lisp-produced machine bytes through a semantics-blind host mechanism.

**Architecture:** Keep language meaning in `semantic-registry.lisp`; move ISA identity to hardware facts represented as Lisp data; keep CML as optimizer rather than ISA owner. Rust host may allocate/protect/invoke memory but must not know that semantic ID `0104` means addition or choose `ADD` for Lisp.

**Tech Stack:** my-lisp, Rust test harness, x86-64 machine encoding, Linux x86-64 native witness for the first executable slice.

**Spec:** `docs/superpowers/specs/2026-09-14-vertical-lisp-isa-design.md`

## Global Constraints

- `lib/surface/semantic-registry.lisp` remains the sole language-semantic authority.
- Machine facts must never allocate semantic IDs.
- No external textual assembler is allowed in the proof path.
- Lisp owns encoding for proof instructions.
- Host code owns only executable-memory/call mechanism.
- Keep W^X: write first, then execute; no permanent RWX mapping.
- The first proof subset is deliberately `MOV`, `ADD`, `RET`; no whole-ISA population before the vertical slice works.

---

### Task 1: Replace compiler-owned machine authority with three-authority boundary

**Files:**
- Modify: `machine-lowering-boundary.lisp`
- Modify: `crates/my-lisp/tests/machine_lowering_boundary.rs`
- Create: `lib/machine/isa/x86-base.lisp`
- Create: `lib/machine/cpu/intel-core-i5-6400.lisp`

**Interfaces:**
- Consumes: current semantic registry and retired ID 1153 rule.
- Produces: machine-readable facts `isa-authority`, `isa-representation`, `instruction-encoding`, `optimization-authority`, plus an ISA catalogue schema containing `MOV`, `ADD`, `RET` and a CPU profile referencing `X86-BASE`.

- [ ] **Step 1: Write the failing boundary/catalogue tests**

Update `machine_lowering_boundary.rs` so it requires:

```rust
for required in [
    "(semantic-authority my-lisp)",
    "(isa-authority hardware-specification)",
    "(isa-source intel-xed/intel-sdm)",
    "(isa-representation my-lisp)",
    "(instruction-encoding my-lisp)",
    "(optimization-authority cml)",
    "(semantic-id-from-isa forbidden)",
    "(raw-execution-mechanism host)",
    "(retired-semantic-id 1153)",
] { /* assert contains */ }
```

Add a test that parses `lib/machine/isa/x86-base.lisp` and requires independent entries for `MOV`, `ADD`, `RET`, each classified under `X86-BASE`, without any semantic ID field.

- [ ] **Step 2: Verify RED**

Run: `cargo test -p my-lisp --test machine_lowering_boundary`

Expected: FAIL because the old boundary still says `machine-instruction-identity compiler-owned` and the new ISA catalogue/profile files do not exist.

- [ ] **Step 3: Implement the minimal boundary/catalogue**

Replace the old authority contract with the three-authority model and create Lisp data files shaped like:

```lisp
(isa-catalogue/1
  (isa x86-64)
  (extensions
    (X86-BASE
      (instruction MOV ...)
      (instruction ADD ...)
      (instruction RET ...))))
```

The CPU profile declares `intel-core-i5-6400`, `skylake`, `x86-64`, and `X86-BASE` support. It must not duplicate language semantics.

- [ ] **Step 4: Verify GREEN**

Run: `cargo test -p my-lisp --test machine_lowering_boundary`

Expected: PASS.

- [ ] **Step 5: Commit**

Commit message: `arch(machine): make ISA hardware facts Lisp-readable`

---

### Task 2: Lisp-owned x86-64 encoder for MOV, ADD, RET

**Files:**
- Create: `lib/machine/encoding/x86-64.lisp`
- Create: `crates/my-lisp/tests/x86_64_lisp_encoder.rs`

**Interfaces:**
- Consumes: `X86-BASE` catalogue facts from Task 1 and ordinary my-lisp exact arithmetic/list operations.
- Produces Lisp functions `x86-reg-code`, `x86-encode-rex`, `x86-encode-modrm`, `x86-encode-mov-r64-imm64`, `x86-encode-add-r64-r64`, `x86-encode-ret`, and `x86-encode-program` returning lists of byte integers.

- [ ] **Step 1: Write failing golden-byte tests**

The Rust test bootstraps core Lisp, evaluates the encoder source, then requires:

```text
(x86-encode-add-r64-r64 'rax 'rbx) => (72 1 216)      ; 48 01 D8
(x86-encode-ret)                     => (195)           ; C3
(x86-encode-mov-r64-imm64 'rax 2)   => (72 184 2 0 0 0 0 0 0 0)
```

It also requires one extended-register case to prove REX register bits are computed rather than hard-coded.

- [ ] **Step 2: Verify RED**

Run: `cargo test -p my-lisp --test x86_64_lisp_encoder`

Expected: FAIL because `lib/machine/encoding/x86-64.lisp` and encoder bindings do not exist.

- [ ] **Step 3: Implement minimal Lisp encoder**

Use arithmetic rather than Rust bit helpers:

```lisp
(def x86-encode-modrm
  (lambda (mode reg rm)
    (+ (* mode 64) (* reg 8) rm)))

(def x86-encode-rex
  (lambda (w r x b)
    (+ 64 (* w 8) (* r 4) (* x 2) b)))
```

Register codes are Lisp data. Immediate encoding uses repeated `mod 256` and `quotient 256` for little-endian bytes. `x86-encode-program` concatenates already encoded instruction byte lists.

- [ ] **Step 4: Verify GREEN**

Run: `cargo test -p my-lisp --test x86_64_lisp_encoder`

Expected: PASS with exact golden bytes.

- [ ] **Step 5: Commit**

Commit message: `feat(machine): encode x86-64 base instructions in Lisp`

---

### Task 3: Native execution witness with semantics-blind host mechanism

**Files:**
- Create: `crates/my-lisp-host/src/native_exec.rs`
- Modify: `crates/my-lisp-host/src/lib.rs`
- Create: `crates/my-lisp-host/tests/native_lisp_bytes.rs`

**Interfaces:**
- Consumes: a proper Lisp list of bytes generated by Task 2.
- Produces: host capability `native-call-u64-raw` on `x86_64 Linux` for the first retained proof. It accepts byte code, maps writable memory, copies bytes, changes it to executable, invokes `extern "C" fn() -> u64`, unmaps it, and returns the result as an exact my-lisp number.

- [ ] **Step 1: Write failing native witness test**

The test installs host capabilities, loads core and Lisp encoder, then evaluates a Lisp program equivalent to:

```lisp
(native-call-u64-raw
  (x86-encode-program
    (list
      (x86-encode-mov-r64-imm64 (quote rax) 2)
      (x86-encode-mov-r64-imm64 (quote rbx) 3)
      (x86-encode-add-r64-r64 (quote rax) (quote rbx))
      (x86-encode-ret))))
```

Expected value: `5`.

The test must also assert that `native-call-u64-raw` is absent from `semantic-registry.lisp` and that Rust source contains no semantic ID `0104` or `додати`/`+` lowering decision.

- [ ] **Step 2: Verify RED**

Run: `cargo test -p my-lisp-host --test native_lisp_bytes`

Expected: FAIL because `native-call-u64-raw` is not installed.

- [ ] **Step 3: Implement minimal Linux x86-64 host mechanism**

Use direct libc FFI declarations local to `native_exec.rs` to avoid a new crate dependency. Sequence: `mmap(PROT_READ|PROT_WRITE)` → copy → `mprotect(PROT_READ|PROT_EXEC)` → call → `munmap`. Never request RWX.

On unsupported targets, do not register the capability; catalogue and encoder tests remain portable.

- [ ] **Step 4: Verify GREEN**

Run: `cargo test -p my-lisp-host --test native_lisp_bytes`

Expected: PASS on x86-64 Linux.

- [ ] **Step 5: Regression verification**

Run:

```text
cargo test -p my-lisp --test machine_lowering_boundary
cargo test -p my-lisp --test x86_64_lisp_encoder
cargo test -p my-lisp-host --test native_lisp_bytes
cargo test -p my-lisp-host
```

Expected: all PASS.

- [ ] **Step 6: Commit**

Commit message: `feat(host): execute Lisp-owned native bytes`

---

### Task 4: Preserve PR #118 projection as a consumer, not authority

**Files:**
- Modify: `lib/machine/intel-core-i5-6400.lisp` or replace it with a compatibility projection if needed.
- Modify: `scripts/generate-function-table.lisp` only if its input path changes.
- Modify: `crates/my-lisp/tests/i5_6400_machine_profile.rs`.
- Modify: `docs/generated/function-table.md` only through the repository generator.

**Interfaces:**
- Consumes: new ISA catalogue + CPU profile.
- Produces: human-readable semantic-to-machine hints without owning ISA or semantic meaning.

- [ ] **Step 1: Write failing projection tests**

Require the generated table to continue showing `0104 / додати` with an i5-6400 path while also asserting that raw ISA entries exist independently without semantic IDs.

- [ ] **Step 2: Verify RED**

Run: `cargo test -p my-lisp --test i5_6400_machine_profile`

Expected: FAIL if the old projection still acts as machine authority rather than consuming the new profile.

- [ ] **Step 3: Refactor projection to consume the new model**

Keep the human column; remove any implication that the 62 mappings define the CPU instruction set.

- [ ] **Step 4: Verify GREEN and generated-file consistency**

Run focused test plus the function-table generator `--check`/equivalent used by CI.

- [ ] **Step 5: Commit**

Commit message: `refactor(machine): make function table consume ISA projection`

## Completion gate

Before claiming the first vertical slice is complete, verify exact-head CI and require these evidence statements only:

1. ISA facts are represented in Lisp independently of semantic IDs.
2. Lisp computes exact x86-64 bytes for the proof forms.
3. Lisp-produced bytes execute natively and return the expected result.
4. Rust host performs only allocation/protection/invocation in the proof path.

Do **not** yet claim full vertical Lisp; semantic lowering of `0104` and structural `car/cdr/cons` are the next evidence stages.
