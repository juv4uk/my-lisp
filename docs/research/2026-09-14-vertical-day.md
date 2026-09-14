# 2026-09-14 — Vertical Day

**Status:** RATIFIED  
**checked-at:** 2026-09-14  
**verified-against main:** `bee413570e4c43c8097d7e6828af2e7665b561f7`  
**research milestone:** #126 `[RATIFIED / VERTICAL DAY] VERTICAL-LISP-2`

> **Не додавати величі, а закривати стрілки доказами.**

## Що саме ратифіковано

2026-09-14 ратифіковано bounded technical claim:

> **Структурна модель даних Lisp, її semantic identity і її фізичне виконання пройшли вертикально до CPU, причому host не отримав права визначати значення CAR/CDR/CONS.**

Це технічний Vertical Day. Це **не** Reproducibility Day і не незалежна наукова ратифікація; наступний шлюз для цього — #131.

## Ратифікований позитивний ланцюжок

```text
(перше (сполучити 2 3))
        ↓
semantic IDs 0005 / 0004
        ↓
Lisp-owned bounded pair representation
        ↓
structured Lisp machine forms
        ↓
closed Lisp-owned admission
        ↓
Lisp-owned x86-64 encoding
        ↓
semantics-blind host
        ↓
physical CPU
        ↓
2
```

Другий witness проходить тим самим шляхом:

```text
(решта (сполучити 2 3)) → ... → CPU → 3
```

Host має право на механізм — raw arena, W^X allocation/copy/protection/call/return — але не на semantic meaning `CONS`, `CAR`, `CDR`.

## Ратифікована негативна стрілка

```text
ill-typed semantic input
or
unadmitted / raw / malformed machine request
        ↓
canonical Type or Lisp-owned named rejection
        ↓
HOST CALL COUNT = 0
```

Негативний доказ не використовує `SIGSEGV`, access violation, illegal instruction або інший host/backend crash як oracle.

## Evidence ledger

### Threshold / machine path

- PR #118 — Lisp-owned machine path, ABI-safe native witness, semantics-blind W^X execution.
- semantic ID `1153` retired; target instructions не мають права народжувати language semantics.
- machine authority визначається subset-властивостями, а не шляхом implementation file.

### Positive structural CPU proof — PR #129

- exact pre-merge head: `2f0de7882761d7f3d502de75ba929d2dc6a98afe`
- focused CI: `34874510826`
- result: core GREEN, `Native host vertical witness` GREEN, changed-surface clippy GREEN
- merged machine slice: `5bafbf9b1f7408c9826b3413f19d50906e9b1867`
- witnesses: interpreter/native `CAR -> 2`, `CDR -> 3`; host remains semantics-blind

### Closed admission — PR #137 / issue #135

- initial RED: `d3be0b6556f0ecd16e445423548591890654c29e`
- RED CI: `34878527752`
- RED result: 8 existing witnesses passed, 1 intended admission failure
- GREEN: closed Lisp-owned structured-form admission
- `UD2` is not special-cased; it is rejected because it is outside the admitted table
- spy counter proves host calls = `0`

### Structured routing — PR #138

- RED: missing `x86-lower-add-u64-forms`, intended failure only
- merged as: `51e95454c9ac492b42b97b139800d517e98d2032`
- canonical route:

```text
semantic lowering
  -> structured machine forms
  -> closed admission
  -> admitted encoder
  -> exact bytes
  -> semantics-blind host
```

Compatibility byte projections may exist temporarily, but route through `x86-encode-admitted-program`; removal/deprecation is tracked separately in issue #139.

### Canonical Type discipline — PR #141 / issue #140

- RED: `4f8c8c943b8a00c6f1817fed687b0e02021dc049`
- RED CI: `34885562086`
- GREEN: `1ead1b14511ac4e92f2c8135889b03c1282821ca`
- GREEN CI: `34886157192`
- merged main commit: `834d037e8c29e179e49ede3225a0eb1f6016fb5e`
- `(car 5)` => canonical `Type`, host calls = `0`
- `(car '())` => canonical `Type`, host calls = `0`
- valid bounded pair still proceeds through structured admitted forms
- full legacy push regression: `34887739113` GREEN

### Adversarial evidence-only closure — PR #142

- production code changes: **none**
- merged as: `4ba4c33dd83c475e162ef88c76495e71eb2b389a`
- focused CI: `34887879362`
- executable adversaries:
  - raw byte data `(15 11)` rejected before host
  - known instruction with unadmitted `rbx` rejected before host
  - truncated admitted-looking form rejected before host
  - host spy remains `0` in every case
- same CI run also executes machine Type discipline and machine-lowering boundary witnesses

This evidence-only PR is significant because the final negative-arrow claim was strengthened by witnesses alone, without changing production semantics to make the test pass.

### CI policy after ratification — PR #143

- current main: `bee413570e4c43c8097d7e6828af2e7665b561f7`
- PR #143 changes CI policy only; no Lisp/Rust semantic change after #142
- current-main fast regression: `34892494227` GREEN in 1m29s
- fast gate retains generated authority checks, Canon/adversarial and SemanticRef fail-closed witnesses, machine lowering/admission/type-discipline, native CPU witness, CLI smoke, `xtask verify`, focused clippy
- full workspace/meta/Android/WASM/browser verification moved to nightly/manual workflows
- superseded main runs may be cancelled so the swarm is not serialized behind stale SHAs

## Claims made

The following are now admissible claims for this bounded slice:

1. semantic identities `0004/0005/0006` are language-owned and do not derive from ISA/opcodes;
2. bounded pair representation and offsets used by the native witness are Lisp-owned machine-readable facts;
3. semantic lowering produces structured forms before byte flattening;
4. only admitted structured forms reach the canonical encoder/executor path;
5. exact x86 bytes are produced by Lisp-owned encoding;
6. host execution is mechanism-only and does not define `CONS/CAR/CDR` meaning;
7. `(перше (сполучити 2 3))` reaches the physical CPU and returns `2`;
8. `(решта (сполучити 2 3))` reaches the physical CPU and returns `3`;
9. ill-typed `CAR` requests preserve canonical `Type` before machine/host entry;
10. raw, malformed and unadmitted machine requests fail closed before host entry.

## Claims **not** made

This record deliberately does **not** claim:

- a complete native Lisp implementation;
- first-class escaping native pairs;
- a general heap/GC design;
- a universal tagged-object ABI;
- a complete x86 assembler/disassembler/verifier;
- arbitrary safe execution of untrusted native code;
- Windows W^X parity;
- Cyberpunk native/JIT completion;
- FPGA/GPU backend parity;
- that compatibility byte wrappers are already removed;
- independent third-person reproduction;
- scientific novelty or world-first status.

The bounded pair proof explicitly keeps `lifetime = native-call` and `escape = forbidden` for this milestone.

## Method demonstrated on this day

The durable result of 2026-09-14 is not only the CPU witness. The project demonstrated a repeatable proof discipline:

- claim first expressed as a falsifiable acceptance boundary;
- RED witness recorded with exact head and intended failure;
- GREEN recorded separately;
- positive and negative arrows proved independently;
- host-call counters used to prove that rejection occurs before the lower layer;
- adversarial evidence can strengthen a claim without changing production code;
- `claims not made` recorded next to claims made;
- semantic authority and mechanism authority kept separate;
- current-state analysis tied to exact SHA/CI evidence rather than recollection.

The practical consequence is that the Vertical Day claim can be reread later as an evidence chain, not reconstructed from chat or author memory.

## Deferred on purpose

The following are explicitly deferred after ratification:

- issue #139 — retirement/deprecation of legacy compatibility byte wrappers;
- Windows W^X executor parity;
- first-class escaping pairs;
- GC/general heap design;
- wider ISA/subset expansion.

None is allowed to retroactively become part of the proof required for this day. Each requires its own future claim and witnesses.

## Next gate — #131 Reproducibility Day

Vertical Day is technical ratification, not independent reproduction.

#131 requires a separate protocol:

```text
clean machine / VM
      ↓
one canonical README entrypoint
      ↓
third person, no author hints
      ↓
<= 30 minutes
      ↓
semantic witness + #126 native witness
      ↓
recorded PASS / recorded friction defects
```

Do not collapse #131 into this record. A human cold-start run is a different epistemic event.

## Closing note

The day began as a threshold: semantic authority had reached the edge of physical execution. It ended only after both directions of the boundary were executable-tested:

```text
admitted meaning -> CPU result
unadmitted request -> rejection before host
```

That two-sided closure is the criterion by which 2026-09-14 is recorded here as **Vertical Day**.
