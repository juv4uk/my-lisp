# ADR-004: Closed McCarthy-7 Core and Stratified Semantic Taxonomy
# ADR-004: Замкнене McCarthy-7 ядро та стратифікована семантична таксономія

**Status:** Accepted / Прийнято  
**Date:** 2026-09-05  
**Author / Authority:** Volodymyr / Vova (Owner directive) & Antigravity  
**Scope:** Normative and documentary only. Zero evaluator/runtime code modifications in this decision.

---

## 1. Context / Контекст

`my-lisp` defines itself as a minimal, homoiconic Lisp that grows from within itself. Historically, dialects of Lisp have suffered semantic bloat when host runtime conveniences, specialized evaluator optimizations, or substrate capabilities silently leaked into the language's core axiomatic definition.

The current Rust implementation in `crates/my-lisp` provides a rich set of facilities (arithmetic, exact rationals, closures, strings, I/O, hashing, JSON, date/time, and vectors). Without a strict normative boundary, it is dangerously easy to conflate everything present in the implementation with the fundamental ontological core of the language.

---

## 2. Normative Decision / Нормативне рішення

### English:
1. **The primitive semantic set of `my-lisp` is permanently closed.** It consists of exactly seven semantic primitives:
   ```text
   { quote, atom, eq, car, cdr, cons, cond }
   ```
2. **No eighth primitive may be admitted.** No implementation, compiler backend, runtime substrate, standard library, or future architectural extension may enlarge this set.
3. **Canonical Conformance Formulation:**
   > *“A conforming implementation may expose many language capabilities, but it shall classify exactly seven operations as semantic primitives: `quote`, `atom`, `eq`, `car`, `cdr`, `cons`, and `cond`. No other capability may acquire primitive status.”*
4. **Implementation convenience cannot create language ontology.** The presence of an instruction in hardware (e.g. FPGA `ADD`, x86 `lea`) or an execution helper in a substrate runtime (e.g. `wsm_add`, `wsm_pci_config_read16`) does not make that operation a semantic primitive of `my-lisp`.
5. **No Self-Proving Introspection:** The language shall **not** define an internal introspection form (such as `primitive-names`). Verification of the closed set belongs exclusively to external conformance test manifests and testing harnesses.
6. **Negative Invariant:** Any implementation manifest or conformance report that includes additional entries (such as `add`, `eval`, `lambda`, or `read`) in the primitive set shall be rejected fail-closed as `PRIMITIVE_SET_VIOLATION`.

### Українська:
1. **Набір семантичних примітивів `my-lisp` є замкненим назавжди.** Він складається рівно із семи семантичних примітивів:
   ```text
   { quote, atom, eq, car, cdr, cons, cond }
   ```
2. **Восьмий примітив не може бути доданий.** Жодна реалізація, компіляторний бекенд, рантайм-субстрат, стандартна бібліотека чи майбутнє розширення не мають права розширювати цей набір.
3. **Канонічне конформне формулювання:**
   > *«Конформна реалізація може надавати багато мовних можливостей, але статус семантичного примітива мають рівно сім операцій: `quote`, `atom`, `eq`, `car`, `cdr`, `cons`, `cond`. Жодна інша можливість не може набути статусу примітива.»*
4. **Зручність реалізації не створює онтологію мови.** Наявність апаратної інструкції (наприклад, FPGA `ADD`, x86 `lea`) або допоміжної функції рантайму субстрату (наприклад, `wsm_add`, `wsm_pci_config_read16`) не робить цю операцію семантичним примітивом `my-lisp`.
5. **Без самостверджувальної інтроспекції:** Мова **не** повинна містити внутрішньої операції переліку примітивів (на кшталт `primitive-names`). Перевірка замкненості множини належить виключно зовнішньому маніфесту відповідності та тестовому harness.
6. **Негативний інваріант:** Будь-який маніфест реалізації чи звіт конформності, який включає додаткові операції (наприклад, `add`, `eval`, `lambda`, `read`) до множини примітивів, бракується за правилом fail-closed із результатом `PRIMITIVE_SET_VIOLATION`.

---

## 3. Strict Non-Equivalence / Строге розрізнення понять

To prevent semantic leakage, `my-lisp` strictly distinguishes the following categories:

```text
SEMANTIC PRIMITIVE
  ≠ ADMITTED DOMAIN SEMANTICS
  ≠ DERIVED FORM
  ≠ LANGUAGE EXTENSION
  ≠ RUNTIME BUILTIN
  ≠ COMPILER INTRINSIC
  ≠ SUBSTRATE ABI FUNCTION
  ≠ HARDWARE OPCODE
```

- **Semantic Primitive (L0):** One of the seven closed McCarthy operations that cannot be reduced without circularity.
- **Admitted Domain Semantics (L1):** First-class value types and representations recognized by the language (e.g. symbols, pairs, exact integers, exact rationals). Arithmetic over these domains is recognized as an admitted domain capability, **not** a silent expansion of the 7 primitives.
- **Derived Form (L2):** A syntax or operation whose entire semantics is defined by expansion/transformation into strictly lower layers, backed by empirical proof.
- **Language Extension (EXT):** An observable language capability admitted as a separate module or platform feature.
- **Runtime Builtin (IMPL):** An optimized host function in an implementation (e.g. Rust function, C subroutine) providing evaluation speed or host interaction.
- **Compiler Intrinsic:** Direct lowering directive inside a compiler (e.g. CML AST to native assembly).
- **Substrate ABI Function:** Freestanding machine interface (e.g. System V AMD64 integer calling convention in `wsm-os-target`).
- **Hardware Opcode:** Execution primitive of physical silicon (e.g. FPGA Verilog state machine or x86 ALU instruction).

---

## 4. Stratified Architecture / Стратифікована архітектура

Substrates do not sit as a language layer. The language exists purely as semantics, observed across independent implementations:

```text
               MY-LISP SEMANTIC PYRAMID
                          │
  ┌──────────────────────────────────────────────┐
  │ L0: CLOSED SEMANTIC CORE                     │
  │     quote, atom, eq, car, cdr, cons, cond    │
  └───────────────────────┬──────────────────────┘
                          ▼
  ┌──────────────────────────────────────────────┐
  │ L1: ADMITTED VALUE DOMAINS                   │
  │     symbols, pairs, exact integers,          │
  │     exact rationals                          │
  └───────────────────────┬──────────────────────┘
                          ▼
  ┌──────────────────────────────────────────────┐
  │ L2: DERIVED LANGUAGE                         │
  │     lambda, def, let, list, cadr, ...        │
  │     (requires verified transformation witness)│
  └───────────────────────┬──────────────────────┘
                          ▼
  ┌──────────────────────────────────────────────┐
  │ L3: KNOWLEDGE / WORLD                        │
  │     SI defining constants (lib/si.my),       │
  │     physics, Panini, reasoning, chess        │
  └──────────────────────────────────────────────┘

────────────────────────────────────────────────────────
SUBSTRATES ARE ORTHOGONAL IMPLEMENTATION OBSERVERS:

                   my-lisp contract
                          │
           ┌──────────────┼──────────────┐
           ▼              ▼              ▼
       Rust core        C core       FPGA fabric
       (software)     (freestanding) (silicon gates)
```

---

## 5. Audit of Current Rust Implementation / Аудит поточної Rust-реалізації

We do not disguise the current state of `crates/my-lisp` by asserting that everything outside the seven primitives is "already proven derived or merely an invisible builtin." Instead, we explicitly classify current visible features, introducing the status `? UNRESOLVED` (semantic debt requiring explicit derivation proof or formal admission):

| Feature / Surface Form | Current Rust Mechanism | Semantic Status in `my-lisp` | Note / Rationale |
| :--- | :--- | :--- | :--- |
| `quote` | special form (`eval/mod.rs`) | **L0 PRIMITIVE** | McCarthy-7 core |
| `atom` | builtin (`eval/builtins.rs`) | **L0 PRIMITIVE** | McCarthy-7 core |
| `eq` | builtin (`eval/builtins.rs`) | **L0 PRIMITIVE** | McCarthy-7 core |
| `car` | builtin (`eval/builtins.rs`) | **L0 PRIMITIVE** | McCarthy-7 core |
| `cdr` | builtin (`eval/builtins.rs`) | **L0 PRIMITIVE** | McCarthy-7 core |
| `cons` | builtin (`eval/builtins.rs`) | **L0 PRIMITIVE** | McCarthy-7 core |
| `cond` | special form (`eval/mod.rs`) | **L0 PRIMITIVE** | McCarthy-7 core |
| `symbols` | `Value::Symbol`, interning | **L1 ADMITTED DOMAIN** | Admitted atomic value |
| `pairs` | `Value::Pair` | **L1 ADMITTED DOMAIN** | Compound data domain |
| `exact integers` | `BigInt` (`bignum.rs`) | **L1 ADMITTED DOMAIN** | Admitted numeric domain |
| `exact rationals` | `Rational` (`bignum.rs`) | **L1 ADMITTED DOMAIN** | Admitted numeric domain |
| `+`, `-`, `*`, `/` | builtin / `arithmetic.rs` | **? UNRESOLVED** | Distinct domain capability vs primitive leak |
| `<`, `>`, `=` | builtin / `arithmetic.rs` | **? UNRESOLVED** | Predicates over numbers |
| `lambda` | special form (`closures.rs`)| **? UNRESOLVED** | Evaluator constructs closure directly; derivation unproven |
| `def` | special form (`eval/mod.rs`) | **? UNRESOLVED** | Environment binding form |
| `defmacro` | special form (`eval/mod.rs`) | **? UNRESOLVED** | Meta-programming mechanism |
| `eval` | special form (`eval/mod.rs`) | **? UNRESOLVED** | Evaluator reflection |
| `print`, `princ` | special form (`eval/mod.rs`) | **? UNRESOLVED** | Host I/O side effect |
| `read`, `read-all` | special form (`eval/mod.rs`) | **? UNRESOLVED** | Host reader reflection |
| `strings` family | special forms & `Value::String` | **? UNRESOLVED** | Text domain vs pair encoding |
| `vectors` family | builtins (`eval/builtins.rs`) | **? UNRESOLVED** | Random-access array domain |
| `sha256-hex` | special form (`eval/mod.rs`) | **? UNRESOLVED** | Host cryptographic utility |
| `json-parse` | special form (`eval/mod.rs`) | **? UNRESOLVED** | Host data interchange utility |
| `utc-now`, `ntp` | builtins (`eval/builtins.rs`) | **? UNRESOLVED** | Host capability / wall-clock service |

---

## 6. The Evidence Requirement for "Derived" / Вимога доказу для статусу «Derived»

**"Derived is a claim requiring evidence."**  
An operation or syntactic form cannot be declared `L2 DERIVED` simply because it is desirable or conceptually elegant. A valid derivation requires an executable witness:
1. **Explicit Transformation / Definition:** The form must be expressed solely in terms of admitted lower layers (L0 primitives and L1 admitted domains).
2. **Behavioral & Semantic Parity:** An automated conformance test must prove that the derived expression satisfies all contract obligations identically to any reference engine.
3. **No Hidden Evaluator Intrinsics:** The evaluator must not rely on unadmitted internal hooks or bypasses to execute the form.

---

## 7. Consequences / Наслідки

1. **Freedom from Ontological Creep:** Future agents, implementers, and backends are prohibited from promoting convenience operations (such as `add` or `eval`) to primitive status.
2. **Honest Roadmap for Simplification:** The `? UNRESOLVED` column identifies the exact semantic debt of the Rust implementation. Subsequent milestones will systematically resolve each `?` into either an admitted value domain, a proven derived form, an external capability, or a cleanly sequestered host tool.
3. **Substrate Neutrality:** `my-lisp` remains an autonomous mathematical creation that presides above Rust, C, and FPGA silicon.
