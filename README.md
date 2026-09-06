# my-lisp

**A small language that grows itself · Маленька мова, що вирощує себе · Eine kleine Sprache, die sich selbst wachsen lässt**

`my-lisp` is a Lisp research language built around a deliberately small semantic nucleus, exact arithmetic, executable conformance, and one architectural rule: **if behavior can be derived inside the language, it must justify why it still lives in the host**.

`my-lisp` is the project name. The separate repository `juv4uk/wsm` is unrelated foundational research. The current canonical source extension here is **`.wsm`**; **`.my`** and **`.lisp`** remain fully supported aliases.

## Quick try · Швидко спробувати · Schnell ausprobieren

The release workflow publishes a standalone browser REPL, and the native CLI can be built directly:

```bash
cargo run -p my-lisp-cli
cargo run -p my-lisp-cli -- path/to/file.wsm
```

Build and test the whole workspace with:

```bash
cargo build --workspace
cargo test --workspace
```

## Semantic authority

The Rust implementation is the **reference implementation**, not the owner of language semantics.

Authority is intentionally ordered:

```text
language-contract.my
        ↓
ratified ADRs
        ↓
executable conformance fixtures / language-owned laws
        ↓
reference implementation (Rust)
        ↓
independent substrates
        ↓
generated reference
        ↓
README / tutorials / historical plans
```

See [`docs/semantic-authority-map.md`](docs/semantic-authority-map.md). If explanatory prose conflicts with a higher-level contract source, the prose is stale.

## Closed semantic core

The primitive semantic operation set is permanently closed:

```text
quote · atom · eq · cons · car · cdr · cond
```

The concrete empty proper list `()` is treated as Canon 0: a value/syntax identity and the inductive base of proper lists, **not an eighth operation**.

The implementation also contains a small evaluator/bootstrap substrate for closures, definitions, macro bootstrapping, exact numbers, structured errors, and host capabilities. Those mechanisms are not allowed to silently expand the seven-operation semantic primitive set.

The current bootstrap work deliberately distinguishes:

```text
semantic primitive identity
≠ evaluator mechanism
≠ surface spelling
≠ derived library form
```

For example, `lib/macro.my` now owns the normal `defmacro` binding after the macro layer loads, while a Rust fallback remains during migration. Observable contract changes still go through `language-contract.my`; implementation refactoring does not rewrite the contract by implication.

## The host boundary

The project does not pursue “rewrite Rust in Lisp” as a goal. It separates **external mechanism** from **language meaning**.

```text
OS / hardware
    ↓
host observations and capabilities
    ↓
my-lisp values
    ↓
Lisp-owned interpretation / policy / protocol
```

Recent examples:

```text
Rust: mono-ns
Lisp: mono-ms, elapsed time, deadline arithmetic

Rust: unix-time-now
Lisp: Gregorian conversion, UTC interpretation, utc-now

Rust: raw NTP/network observation
Lisp: timestamp meaning and synchronization policy
```

The living audit is [`docs/host-semantic-surface.md`](docs/host-semantic-surface.md).

## Exact arithmetic

Exactness is a core design choice, not a display preference. Integer and rational arithmetic remains exact instead of silently becoming floating point. The Rust runtime supplies the low-level arbitrary-precision machinery; language semantics decide what exact results mean.

This project prefers measurable costs over hidden compromises: performance regressions caused by stronger exactness or stack-safety guarantees are documented and tested rather than disguised.

## A language that grows itself

The slogan is tested by executable code, not only by prose. The repository contains substantial systems written in my-lisp itself:

- [`lib/core.my`](lib/core.my) — bootstrapped standard library;
- [`lib/macro.my`](lib/macro.my) — language-owned macro bootstrap layer;
- [`lib/canon.my`](lib/canon.my) — executable Canon 0+7 semantic laws;
- [`lib/meta-eval.my`](lib/meta-eval.my) — metacircular evaluation experiments;
- [`lib/unify.my`](lib/unify.my), [`lib/reason.my`](lib/reason.my), [`lib/forward.my`](lib/forward.my) — symbolic reasoning;
- [`lib/knowledge.my`](lib/knowledge.my), [`lib/world.my`](lib/world.my), [`lib/content-store.my`](lib/content-store.my), [`lib/epistemic.my`](lib/epistemic.my) — knowledge/history/epistemic layers;
- [`lib/understand.my`](lib/understand.my), [`lib/narrate.my`](lib/narrate.my) — controlled language bridges;
- [`lib/clips-import.my`](lib/clips-import.my) — CLIPS import;
- [`lib/time.my`](lib/time.my) — increasingly language-owned time semantics.

The scientific question is therefore not “how many lines are Lisp?” but:

> **How small can the irreducible host remain while the useful system continues to grow inside the language?**

## Implementations and tools

This repository contains the mature software reference and tooling:

- [`crates/my-lisp`](crates/my-lisp) — parser, evaluator, values, exact arithmetic, environments, diagnostics;
- [`crates/my-lisp-cli`](crates/my-lisp-cli) — CLI / REPL;
- [`crates/my-lisp-wasm`](crates/my-lisp-wasm) — WebAssembly bindings;
- [`crates/my-lisp-literate`](crates/my-lisp-literate) — literate source mapping;
- [`crates/my-lisp-lsp`](crates/my-lisp-lsp) — LSP adapter;
- [`crates/my-lisp-host`](crates/my-lisp-host) — explicit OS capability layer;
- [`crates/my-lisp-semantic`](crates/my-lisp-semantic) — experimental Sanskrit/Pāṇinian semantic research;
- [`crates/swarm-node`](crates/swarm-node) and Guard crates — ecosystem experiments built around the same mechanism/policy separation;
- [`racket/`](racket/) — `#lang my-lisp` support for Racket/DrRacket;
- [`c-runtime/`](c-runtime/) — reinstated C + x86_64 assembly substrate.

A physically different HDL Lisp-machine implementation lives in the separate [`fpga-lisp`](https://github.com/juv4uk/fpga-lisp) repository. Independent substrates exist to **falsify implementation-specific assumptions**, not to share one implementation architecture.

## Research scope

The repository is intentionally more than a toy interpreter, but the layers have different status:

```text
L0  runtime/substrate mechanisms
L1  closed semantic core
L2  bootstrap + standard language
L3  semantic libraries
L4  reasoning
L5  knowledge/history
L6  explicit host capabilities
L7  applications and ecosystem experiments
```

Not every experiment is part of the language contract. In particular, Sanskrit/Pāṇinian work, Guard, swarm coordination, natural-language bridges, and agent experiments must not silently acquire primitive-language status merely because they are in the same repository.

## Documentation

Start here:

- [`language-contract.my`](language-contract.my) — machine-readable Level 1/2 semantic-contract version;
- [`docs/semantic-authority-map.md`](docs/semantic-authority-map.md) — what outranks what when documents disagree;
- [`docs/language-core.md`](docs/language-core.md) — compact human-readable core architecture;
- [`docs/host-semantic-surface.md`](docs/host-semantic-surface.md) — host/Lisp ownership inventory;
- [`docs/adr/ADR-004-CLOSED-MCCARTHY7-CORE.md`](docs/adr/ADR-004-CLOSED-MCCARTHY7-CORE.md) — ratified closed seven-operation core;
- [`tests/fixtures/conformance.my`](tests/fixtures/conformance.my) — executable conformance fixtures;
- [`docs/FUNCTIONS.md`](docs/FUNCTIONS.md) — generated function/builtin reference;
- [`docs/testing.md`](docs/testing.md) — test inventory;
- [`docs/benchmarks.md`](docs/benchmarks.md) — benchmark methodology;
- [`docs/mccarthy-vision.md`](docs/mccarthy-vision.md) — historical grounding and explicit departures;
- [`docs/versioning.md`](docs/versioning.md) — project/version history.

Historical decisions, dated audits, `PLAN.md`, and agent notes remain useful evidence of how the project evolved, but they are not allowed to override the current semantic authority chain.

---

## Українською

`my-lisp` — дослідницька Lisp-мова з навмисно малим семантичним ядром, точною арифметикою й виконуваним контрактом сумісності.

Семантичні примітиви назавжди замкнені:

```text
quote · atom · eq · cons · car · cdr · cond
```

`() ` — Canon 0, конкретний порожній правильний список; це значення, а не восьма операція.

Rust є **референсною реалізацією**, а не джерелом семантичної істини. Джерело істини — машинний контракт, ратифіковані рішення та виконувані conformance-тести. Якщо поведінку можна вивести всередині Lisp, вона має пояснити, навіщо лишається в host.

Поточний канонічний суфікс файлів — **`.wsm`**; **`.my`** та **`.lisp`** лишаються повністю підтримуваними. Назва проєкту — **`my-lisp`**; окремий репозиторій `wsm` не перейменовує цю мову.

Найважливіша архітектурна межа:

```text
host дає спостереження / ефект
Lisp визначає значення / політику
```

Саме тому `mono-ms` уже виведений із `mono-ns` у Lisp, а `utc-now` переходить на Lisp-інтерпретацію сирого `unix-time-now`.

Головне дослідницьке питання проєкту: **якою мінімальною може бути незвідна host-машина, якщо решта корисної системи вирощується самою мовою?**

---

## Deutsch

`my-lisp` ist eine Lisp-Forschungssprache mit bewusst kleinem semantischem Kern, exakter Arithmetik und ausführbarer Konformität.

Der semantische Primitivsatz ist dauerhaft geschlossen:

```text
quote · atom · eq · cons · car · cdr · cond
```

`() ` ist Canon 0, die konkrete leere richtige Liste; kein achtes Primitiv.

Rust ist die **Referenzimplementierung**, nicht die semantische Autorität. Autorität liegt beim maschinenlesbaren Vertrag, ratifizierten Entscheidungen und ausführbaren Konformitätstests. Ableitbare Bedeutung soll in Lisp leben; der Host bleibt für echte Beobachtungen und Fähigkeiten zuständig.

Die aktuelle kanonische Quelldateiendung ist **`.wsm`**; **`.my`** und **`.lisp`** bleiben vollständig unterstützt. Der Projektname bleibt **`my-lisp`**.

## License · Ліцензія · Lizenz

[ВОЛЬНІСТЬ](LICENSE)
