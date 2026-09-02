# wsm (my-lisp)

**A small language that grows itself · Маленька мова, що вирощує себе · Eine kleine Sprache, die sich selbst wachsen lässt**

[English](#english) · [Українська](#українська) · [Deutsch](#deutsch)

> **Renamed 2026-08-27 (`ECO-DECISION-2026-08-27-MYLISP-WSM-RENAME`): the
> project is now `wsm` — a different, pre-existing "MyLisp" project
> already exists, and the old name invited real confusion between the
> two. `.wsm` is the new canonical source extension; `.my` and `.lisp`
> remain fully supported, not deprecated. Only this README reflects the
> new name so far — every other doc, comment, path, and crate name in
> this repo (and across the ecosystem: my-lisp-panini, chess-lisp-zero,
> fpga-lisp, cml) still says "my-lisp" on purpose. That is not drift to
> fix on sight: the rename decision was deliberately scoped to name +
> extension only, with repo/crate/doc migration left for a later,
> separately scoped pass. Don't rename anything else on the strength of
> this note alone.**
>
> **Перейменовано 2026-08-27 (`ECO-DECISION-2026-08-27-MYLISP-WSM-RENAME`):
> проєкт тепер називається `wsm` — уже існує інший, чужий проєкт
> "MyLisp", і стара назва провокувала реальну плутанину. `.wsm` — нове
> канонічне розширення, `.my`/`.lisp` лишаються повністю підтримуваними,
> не депрекейтяться. Наразі нову назву відображає лише цей README — уся
> решта документації, коментарів, шляхів і назв crate'ів у цьому репо
> (і в екосистемі: my-lisp-panini, chess-lisp-zero, fpga-lisp, cml)
> навмисно й далі каже "my-lisp". Це не дрейф, який треба виправляти —
> рішення про перейменування свідомо обмежене назвою й розширенням,
> міграція репо/crate'ів/документації лишена на окремий, майбутній крок.
> Не перейменовуй нічого іншого лише на підставі цієї нотатки.**

## Quick try · Швидко спробувати · Schnell ausprobieren

No installation, no account: **[download the standalone web REPL](https://github.com/juv4uk/my-lisp/releases/latest/download/my-lisp-cli-web.html)** — a single portable `.html` file with a terminal-style REPL for the my-lisp core, running entirely in your browser. See [`docs/quote-tutorial.md`](docs/quote-tutorial.md) for a first walkthrough.

Без встановлення й облікового запису: **[завантажте автономний web-REPL](https://github.com/juv4uk/my-lisp/releases/latest/download/my-lisp-cli-web.html)** — один portable-файл `.html` з термінальним REPL для ядра my-lisp, що працює повністю в браузері. Перший огляд — у [`docs/quote-tutorial.md`](docs/quote-tutorial.md).

Ohne Installation und Benutzerkonto: **[den eigenständigen Web-REPL herunterladen](https://github.com/juv4uk/my-lisp/releases/latest/download/my-lisp-cli-web.html)** — eine einzelne portable `.html`-Datei mit einem terminalartigen REPL für den my-lisp-Kern, die vollständig im Browser läuft. Ein erster Rundgang steht in [`docs/quote-tutorial.md`](docs/quote-tutorial.md).

Or build the native CLI from source:

```bash
cargo run -p my-lisp-cli
cargo run -p my-lisp-cli -- path/to/file.wsm
```

## English

`my-lisp` is a Lisp built around McCarthy's seven primitives — `quote`, `atom`, `eq`, `car`, `cdr`, `cons`, `cond` — plus the minimal semantic kernel needed to bootstrap everything else: `lambda`, `def`, `defmacro`. Everything derivable from that kernel is written in my-lisp itself ([`lib/core.my`](lib/core.my)), not added as Rust built-ins. Full rationale: [`docs/language-core.md`](docs/language-core.md).

Exact rational arithmetic is a core purpose, not a nice-to-have: `/` on integers/rationals stays exact (`5/336`, not `0.0148...`), following Racket's exact/inexact distinction. `.wsm` is the canonical source extension; `.my` and `.lisp` remain fully supported aliases, not deprecated.

The central architecture is one structural language for programs, facts, rules, and proofs. Machines exchange knowledge rather than executable commands, then reason locally under the same conformance contract across Rust and FPGA implementations.

### Philosophy

Rust provides only what it does exceptionally well — safe values, parsing, lexical closures, deterministic evaluation, stack control, diagnostics — and stops there. Every derived form grows in my-lisp whenever the kernel can express it: `<=`/`>=` have moved out of Rust dispatch into recursive `lib/core.my`; `eval` reuses the data→code conversion already needed by macro expansion. This is the same instinct that kept John McCarthy from finishing M-expressions once S-expressions turned out to be enough.

That instinct is tested, not just stated: [`lib/meta-eval.my`](lib/meta-eval.my) is a metacircular evaluator — `eval`/`apply` written in my-lisp itself, the same relationship McCarthy's own 1960 paper had to its primitives, dispatching to `car`/`cdr`/`cons`/`atom`/`eq` rather than reimplementing them. [`lib/unify.my`](lib/unify.my) and [`lib/reason.my`](lib/reason.my) provide a small unification and backward-chaining reasoning engine in the spirit of McCarthy's 1958 "Advice Taker" proposal — an inference machine capable of symbolic logic, not statistics. Both are proof that "a small language that grows itself" is a working claim about this codebase, not a slogan.

### Advice Taker: both directions of inference, and both directions of language

[`lib/forward.my`](lib/forward.my) is the other classic half of symbolic AI: a CLIPS-style forward-chaining engine (working memory, fixpoint derivation, `not`/`or`/`and`/`test` conditions) sharing `lib/reason.my`'s exact `(head cond1 cond2 ...)` rule shape — the same rule literal runs backward or forward unmodified. Both engines carry real truth maintenance: single- and multi-justification JTMS, where retracting a fact cascades only to what actually depended on it. [`lib/knowledge.my`](lib/knowledge.my) groups rules into named, queryable modules; [`lib/clips-import.my`](lib/clips-import.my) reads genuine CLIPS `.clp` source files off disk — `deffacts`, `defrule`, `deftemplate`'s named slots, `?x` variables — and imports them as ordinary my-lisp knowledge, proving old symbolic-AI systems can be reused, not just studied.

Explainability runs through all of it, not bolted on after: `explain-proof`/`reason-explain`/`provenance` turn a proof tree into a human-readable "why," and [`lib/understand.my`](lib/understand.my)/[`lib/narrate.my`](lib/narrate.my) bridge controlled natural language to and from knowledge structures — text in, structure out, and back again.

[`tests/fixtures/conformance.my`](tests/fixtures/conformance.my) and [`tests/fixtures/macro-conformance.my`](tests/fixtures/macro-conformance.my) exist because Lisp's history is also a history of dialects drifting apart — MacLisp, InterLisp, a dozen Scheme variants, Common Lisp's attempt to reunify them. This file is the one thing `fpga-lisp`'s HDL Lisp-machine core must agree with, so a second implementation never becomes "just another dialect."

This repository is the canonical Rust implementation:

- [`crates/my-lisp`](crates/my-lisp) — the core: parser, evaluator, environments, exact-rational arithmetic.
- [`crates/my-lisp-cli`](crates/my-lisp-cli) — the `my-lisp` binary (REPL + file runner).
- [`crates/my-lisp-wasm`](crates/my-lisp-wasm) — WebAssembly bindings powering the browser REPL above.
- [`crates/my-lisp-literate`](crates/my-lisp-literate) — literate-Markdown source-offset mapping.
- [`crates/my-lisp-lsp`](crates/my-lisp-lsp) — Language Server Protocol adapter over the canonical parser (diagnostics, document symbols, hover, same-file go-to-definition); also exposed as `my-lisp lsp`.
- [`crates/my-lisp-host`](crates/my-lisp-host) — the OS capability layer (filesystem, subprocess, TCP), installed into the core's capability registry only when an embedder opts in; the core crate itself has zero OS access.
- [`crates/my-lisp-semantic`](crates/my-lisp-semantic) — EXPERIMENTAL: a Sanskrit/Pāṇinian semantic layer (transliteration, the Semantic Atom Registry, the twelve dhātu roots, kāraka roles), not yet wired into the parser/evaluator.
- [`crates/wsm-guard-core`](crates/wsm-guard-core) — the single shared embed of `lib/core.my` + `lib/guard.wsm` and the evaluate-and-validate path every Guard consumer uses.
- [`crates/wsm-guard-slice`](crates/wsm-guard-slice) — a minimal event-driven Rust mechanism that frames one bounded event per line and lets WSM policy own the decision.
- [`crates/wsm-guard-facts`](crates/wsm-guard-facts) — a bounded, read-only fact adapter normalizing Git/systemd/swarm observations into `(fact ...)` clauses for Guard policy to classify.
- [`crates/swarm-node`](crates/swarm-node) — standalone swarm coordination node.
- [`racket/`](racket/) — a `#lang my-lisp` plugin for Racket/DrRacket using the Chez Scheme JIT.
- [`lib/core.my`](lib/core.my) — the bootstrapped standard library.
- [`lib/unify.my`](lib/unify.my) / [`lib/reason.my`](lib/reason.my) — unification and backward-chaining inference.
- [`lib/forward.my`](lib/forward.my) — forward-chaining inference with truth maintenance.
- [`lib/knowledge.my`](lib/knowledge.my) — named, queryable knowledge modules.
- [`lib/world.my`](lib/world.my) — navigable and branch-comparable immutable history, snapshot-local reasoning, atomic ingestion, and data-only exchange.
- [`lib/content-store.my`](lib/content-store.my) — an immutable content-addressed store that deduplicates equal knowledge and World histories.
- [`lib/understand.my`](lib/understand.my) / [`lib/narrate.my`](lib/narrate.my) — controlled natural language, both directions.
- [`lib/clips-import.my`](lib/clips-import.my) — imports real CLIPS `.clp` source files.
- [`lib/yantra.my`](lib/yantra.my) — EXPERIMENTAL: the smallest coding agent whose control logic lives entirely in my-lisp (tool-call loop against an OpenAI-compatible endpoint; see `docs/yantra-agent.md`).

### Build and test

```bash
cargo build --workspace
cargo test --workspace
```

### Racket / DrRacket support

The [`racket/`](racket/) directory contains a `#lang my-lisp` plugin for
Racket (Chez Scheme backend). It matches the core semantics of the Rust
implementation: explicit `(quote x)` (apostrophe is part of symbols),
exact decimal literals, `t`/`()` truth values, exact division, and the
seven McCarthy primitives. Install it locally with:

```sh
raco pkg install --link --name my-lisp racket/
```

Then open any `.my` file starting with `#lang my-lisp` in DrRacket and
press Run, or run it from the terminal with `racket file.my`. See
[`racket/README.md`](racket/README.md) for details.

### Docs

- [`docs/language-core.md`](docs/language-core.md) — the language contract: primitives, bootstrap boundary, exact arithmetic.
- [`docs/FUNCTIONS.md`](docs/FUNCTIONS.md) — a generated reference of every built-in and `lib/*.my` function, one section per library.
- [`docs/DATE-TIME-AND-SYNC-ARCHITECTURE-2026-08-31.md`](docs/DATE-TIME-AND-SYNC-ARCHITECTURE-2026-08-31.md) — date/time interfaces, NTP observation, timezone boundaries, filesystem timestamps, and Guard synchronization.
- [`docs/quote-tutorial.md`](docs/quote-tutorial.md) — a beginner walkthrough of homoiconicity.
- [`docs/advice-taker.md`](docs/advice-taker.md) — a tutorial on building and using the Advice Taker backward-chaining engine.
- [`docs/advice-ingestion.md`](docs/advice-ingestion.md) — the guarded `understand → advise → reason → narrate` knowledge boundary.
- [`docs/knowledge-package-format.md`](docs/knowledge-package-format.md) — versioned data-only interchange for other projects and AI adapters.
- [`docs/content-identity.md`](docs/content-identity.md) — canonical knowledge/world identity before cryptographic hashing.
- [`docs/canonical-serialization.md`](docs/canonical-serialization.md) — the portable `write-to-string` wire format and its round-trip law.
- [`docs/ecosystem-roadmap.md`](docs/ecosystem-roadmap.md) — the contract path across my-lisp, the CML compiler, and fpga-lisp.
- [`docs/clean-code.md`](docs/clean-code.md) / [`CLEAN_CODE_PLAN.md`](CLEAN_CODE_PLAN.md) — Clean Code design principles and their executable roadmap.
- [`docs/unify-tutorial.md`](docs/unify-tutorial.md) — a small symbolic-AI example: unification, written in my-lisp itself.
- [`docs/mccarthy-vision.md`](docs/mccarthy-vision.md) — how John McCarthy himself described Lisp's origin and evolution, from 1958 to his death in 2011, and where this project follows or departs from that account.
- [`docs/testing.md`](docs/testing.md) — current test inventory.
- [`docs/benchmarks.md`](docs/benchmarks.md) — benchmark methodology and a local baseline.
- [`docs/versioning.md`](docs/versioning.md) — why this repo's version history looks the way it does.

`my-lisp` began inside a broader IDE project, [`my-idea`](https://github.com/juv4uk/my-idea), and was extracted here to stand on its own. A second implementation of the same language — a from-scratch Lisp-machine HDL core, `fpga-lisp` — is developed as a separate, parallel repository. A previously-planned third implementation (a C core for embedded targets) was dropped by explicit decision; two independent implementations is the current commitment.

A fifth sibling repository, [`my-lisp-panini`](https://github.com/juv4uk/my-lisp-panini), researches Pāṇini's Sanskrit grammar (*Aṣṭādhyāyī*) as a formal system in its own right, producing the `panini-foundation` that this repo's own Sanskrit semantic-atom migration (`docs/sanskrit-semantic-migration.md`, the `SANSKRIT-P*` tasks) draws on. It does not touch `my-lisp` until its own machine-model gate review is complete — see that repo's `AGENTS.md` for the full research mandate.

## Українська

`my-lisp` — це Lisp, побудований навколо семи примітивів Маккарті — `quote`, `atom`, `eq`, `car`, `cdr`, `cons`, `cond` — плюс мінімальне семантичне ядро, потрібне для розгортання всього іншого: `lambda`, `def`, `defmacro`. Усе, що можна вивести з цього ядра, написане самою my-lisp ([`lib/core.my`](lib/core.my)), а не додане як Rust built-in. Повне обґрунтування — [`docs/language-core.md`](docs/language-core.md).

Точна раціональна арифметика — базова мета, а не бонус: `/` над цілими/раціональними лишається точним (`5/336`, а не `0.0148...`), за зразком розрізнення exact/inexact у Racket. `.wsm` — канонічне розширення початкового коду; `.my` і `.lisp` лишаються повністю підтримуваними псевдонімами, не депрекейтяться.

Центральна архітектура — одна структурна мова для програм, фактів, правил і доведень. Машини обмінюються знаннями, не командами на виконання, а потім міркують локально за спільним conformance-контрактом Rust- і FPGA-реалізацій.

### Філософія

Rust надає лише те, що робить особливо добре — безпечні значення, парсинг, лексичні замикання, детерміноване обчислення, контроль стека й діагностику. Похідні форми ростуть у my-lisp, коли ядро вже може їх виразити: `<=`/`>=` перенесені з Rust-dispatch у рекурсивний `lib/core.my`, а `eval` перевикористовує наявне перетворення дані→код. Це той самий інстинкт, що не дав Маккарті добудувати M-expressions, коли S-виразів виявилось досить.

Цей інстинкт перевірений, не лише задекларований: [`lib/meta-eval.my`](lib/meta-eval.my) — метациркулярний evaluator: `eval`/`apply`, написані самою my-lisp, те саме відношення, яке власна стаття Маккарті 1960 року мала до своїх примітивів — диспетчеризує до `car`/`cdr`/`cons`/`atom`/`eq`, а не переписує їх заново. [`lib/unify.my`](lib/unify.my) і [`lib/reason.my`](lib/reason.my) — маленький механізм унікації й backward-chaining рушій міркування в дусі пропозиції "Advice Taker" Маккарті 1958 року — символьне зіставлення з шаблоном, не статистика. Обидва — доказ того, що "маленька мова, що вирощує себе" — робоче твердження про цей код, не гасло.

### Advice Taker: обидва напрямки висновування, і обидва напрямки мови

[`lib/forward.my`](lib/forward.my) — інша класична половина символьного AI: forward-chaining рушій у стилі CLIPS (working memory, виведення до fixpoint, умови `not`/`or`/`and`/`test`), що ділить із `lib/reason.my` точно ту саму форму правил `(head cond1 cond2 ...)` — той самий літерал правила спрацьовує і назад, і вперед без жодної зміни. Обидва рушії мають справжню truth maintenance: JTMS з одним і з множинними обґрунтуваннями, де видалення факту каскадує лише на те, що справді від нього залежало. [`lib/knowledge.my`](lib/knowledge.my) групує правила в іменовані, запитувані модулі; [`lib/clips-import.my`](lib/clips-import.my) читає справжні CLIPS `.clp`-файли з диска — `deffacts`, `defrule`, іменовані слоти `deftemplate`, змінні `?x` — і імпортує їх як звичайне знання my-lisp, доводячи, що старі символьні AI-системи можна перевикористати, не лише вивчати.

Пояснюваність проходить крізь усе це, не приліплена окремо: `explain-proof`/`reason-explain`/`provenance` перетворюють дерево доведення на людське "чому", а [`lib/understand.my`](lib/understand.my)/[`lib/narrate.my`](lib/narrate.my) з'єднують контрольовану природну мову зі структурами знань в обидва боки — текст на вхід, структура на вихід, і назад.

[`tests/fixtures/conformance.my`](tests/fixtures/conformance.my) і [`tests/fixtures/macro-conformance.my`](tests/fixtures/macro-conformance.my) існують, бо історія Lisp — це також історія діалектів, що розбігались — MacLisp, InterLisp, десяток варіантів Scheme, спроба Common Lisp їх возз'єднати. Цей файл — те єдине, з чим має погоджуватись HDL-ядро Lisp-машини `fpga-lisp`, щоб друга реалізація ніколи не стала "ще одним діалектом".

Цей репозиторій — канонічна реалізація на Rust:

- [`crates/my-lisp`](crates/my-lisp) — ядро: парсер, обчислювач, середовища, точна раціональна арифметика.
- [`crates/my-lisp-cli`](crates/my-lisp-cli) — бінарник `my-lisp` (REPL + запуск файлів).
- [`crates/my-lisp-wasm`](crates/my-lisp-wasm) — WebAssembly-біндінги для браузерного REPL вище.
- [`crates/my-lisp-literate`](crates/my-lisp-literate) — зіставлення зміщень початкового коду literate-Markdown.
- [`crates/my-lisp-lsp`](crates/my-lisp-lsp) — адаптер протоколу Language Server над канонічним парсером (діагностика, символи документа, hover, перехід до визначення в межах файла); також доступний як `my-lisp lsp`.
- [`crates/my-lisp-host`](crates/my-lisp-host) — шар OS-можливостей (файлова система, subprocess, TCP), встановлюється в capability-реєстр ядра лише коли embedder свідомо це вмикає; саме ядро не має жодного доступу до ОС.
- [`crates/my-lisp-semantic`](crates/my-lisp-semantic) — ЕКСПЕРИМЕНТАЛЬНО: санскритсько-паніянівський семантичний шар (транслітерація, Semantic Atom Registry, дванадцять коренів dhātu, ролі kāraka), ще не підключений до парсера/evaluator'а.
- [`crates/wsm-guard-core`](crates/wsm-guard-core) — єдиний спільний embed `lib/core.my` + `lib/guard.wsm` та шлях evaluate-and-validate, яким користується кожен споживач Guard.
- [`crates/wsm-guard-slice`](crates/wsm-guard-slice) — мінімальний event-driven Rust-механізм, що обрамляє одну обмежену подію на рядок і лишає рішення за політикою WSM.
- [`crates/wsm-guard-facts`](crates/wsm-guard-facts) — обмежений read-only адаптер фактів, що нормалізує спостереження Git/systemd/swarm у clause `(fact ...)` для класифікації політикою Guard.
- [`crates/swarm-node`](crates/swarm-node) — окремий вузол swarm-координації.
- [`racket/`](racket/) — плагін `#lang my-lisp` для Racket/DrRacket на базі Chez Scheme JIT.
- [`lib/core.my`](lib/core.my) — bootstrapped стандартна бібліотека.
- [`lib/unify.my`](lib/unify.my) / [`lib/reason.my`](lib/reason.my) — унікація й backward-chaining висновування.
- [`lib/forward.my`](lib/forward.my) — forward-chaining висновування з truth maintenance.
- [`lib/knowledge.my`](lib/knowledge.my) — іменовані, запитувані модулі знань.
- [`lib/world.my`](lib/world.my) — навігована й порівнювана між гілками незмінна історія, snapshot-local reasoning, атомарне надходження та data-only обмін.
- [`lib/content-store.my`](lib/content-store.my) — незмінне content-addressed сховище, що дедуплікує рівні знання та історії World.
- [`lib/understand.my`](lib/understand.my) / [`lib/narrate.my`](lib/narrate.my) — контрольована природна мова в обидва боки.
- [`lib/clips-import.my`](lib/clips-import.my) — імпортує справжні CLIPS `.clp`-файли.

### Збірка та тести

```bash
cargo build --workspace
cargo test --workspace
```

### Підтримка Racket / DrRacket

Каталог [`racket/`](racket/) містить плагін `#lang my-lisp` для Racket (backend
Chez Scheme). Він повторює основну семантику Rust-реалізації: явний
`(quote x)` (апостроф — частина символу), точні десяткові літерали,
істинні значення `t`/`()`, точне ділення та сім примітивів Маккарті.
Локальне встановлення:

```sh
raco pkg install --link --name my-lisp racket/
```

Після цього відкрийте будь-який файл `.my`, що починається з
`#lang my-lisp`, у DrRacket і натисніть Run, або виконайте
`racket file.my` у терміналі. Деталі — у
[`racket/README.md`](racket/README.md).

### Документація

- [`docs/language-core.md`](docs/language-core.md) — контракт мови: примітиви, межа bootstrap, точна арифметика.
- [`docs/FUNCTIONS.md`](docs/FUNCTIONS.md) — згенерований довідник кожної built-in та `lib/*.my`-функції, по розділу на бібліотеку.
- [`docs/DATE-TIME-AND-SYNC-ARCHITECTURE-2026-08-31.md`](docs/DATE-TIME-AND-SYNC-ARCHITECTURE-2026-08-31.md) — повна карта дати/часу, NTP-спостереження, timezone, меж файлової системи та policy синхронізації Guard.
- [`docs/quote-tutorial.md`](docs/quote-tutorial.md) — вступний огляд гомоіконічності.
- [`docs/unify-tutorial.md`](docs/unify-tutorial.md) — маленький приклад символьного AI: унікація, написана самою my-lisp.
- [`docs/advice-ingestion.md`](docs/advice-ingestion.md) — захищена межа знань `understand → advise → reason → narrate`.
- [`docs/knowledge-package-format.md`](docs/knowledge-package-format.md) — версіонований data-only обмін з іншими проєктами й AI-адаптерами.
- [`docs/content-identity.md`](docs/content-identity.md) — канонічна identity знань і світів до криптографічного hashing.
- [`docs/canonical-serialization.md`](docs/canonical-serialization.md) — переносний формат `write-to-string` і його закон round-trip.
- [`docs/ecosystem-roadmap.md`](docs/ecosystem-roadmap.md) — шлях контрактів між my-lisp, компілятором CML і fpga-lisp.
- [`docs/clean-code.md`](docs/clean-code.md) / [`CLEAN_CODE_PLAN.md`](CLEAN_CODE_PLAN.md) — принципи Clean Code та виконуваний roadmap їх упровадження.
- [`docs/mccarthy-vision.md`](docs/mccarthy-vision.md) — як сам Джон Маккарті описував походження й розвиток Lisp, від 1958-го до своєї смерті 2011-го, і де цей проєкт іде за цим викладом, а де відходить.
- [`docs/testing.md`](docs/testing.md) — поточний перелік тестів.
- [`docs/benchmarks.md`](docs/benchmarks.md) — методологія бенчмарків і локальний baseline.
- [`docs/versioning.md`](docs/versioning.md) — чому історія версій цього репо саме така.

`my-lisp` починалась усередині ширшого IDE-проєкту [`my-idea`](https://github.com/juv4uk/my-idea) й була виділена сюди, щоб існувати самостійно. Друга реалізація тієї ж мови — HDL-ядро Lisp-машини з нуля, `fpga-lisp` — розробляється як окремий, паралельний репозиторій. Раніше запланована третя реалізація (C-ядро для embedded-цілей) прибрана свідомим рішенням; дві незалежні реалізації — поточне зобов'язання.

П'ятий сестринський репозиторій, [`my-lisp-panini`](https://github.com/juv4uk/my-lisp-panini), досліджує санскритську граматику Паніні (*Aṣṭādhyāyī*) як формальну систему саму по собі, виробляючи `panini-foundation`, на яку спирається власна санскритська семантична міграція цього репозиторію (`docs/sanskrit-semantic-migration.md`, задачі `SANSKRIT-P*`). Він не торкається `my-lisp`, доки не завершено власний machine-model gate review — повний дослідницький мандат дивись у `AGENTS.md` того репозиторію.

## Deutsch

`my-lisp` ist ein Lisp, aufgebaut um McCarthys sieben Primitive — `quote`, `atom`, `eq`, `car`, `cdr`, `cons`, `cond` — plus den minimalen semantischen Kern, der zum Bootstrap von allem anderen nötig ist: `lambda`, `def`, `defmacro`. Alles, was aus diesem Kern ableitbar ist, ist in my-lisp selbst geschrieben ([`lib/core.my`](lib/core.my)), nicht als Rust-Built-in hinzugefügt. Vollständige Begründung: [`docs/language-core.md`](docs/language-core.md).

Exakte rationale Arithmetik ist ein Kernziel, kein Extra: `/` bleibt bei Ganzzahlen/rationalen Zahlen exakt (`5/336`, nicht `0.0148...`), nach Rackets exakt/inexakt-Unterscheidung. `.wsm` ist die kanonische Quellcodedateiendung; `.my` und `.lisp` bleiben vollständig unterstützte Aliase, nicht veraltet.

Die zentrale Architektur ist eine gemeinsame Struktursprache für Programme, Fakten, Regeln und Beweise. Maschinen tauschen Wissen statt Ausführungsbefehle aus und schließen lokal unter demselben Konformitätsvertrag für Rust- und FPGA-Implementierungen.

### Philosophie

Rust stellt nur bereit, was es besonders gut kann — sichere Werte, Parsing, lexikalische Closures, deterministische Auswertung, Stack-Kontrolle und Diagnosen. Abgeleitete Formen wachsen in my-lisp, sobald der Kern sie ausdrücken kann: `<=`/`>=` sind aus dem Rust-Dispatch in das rekursive `lib/core.my` gewandert; `eval` nutzt die vorhandene Daten→Code-Umwandlung. Derselbe Instinkt ließ McCarthy M-Expressions aufgeben, als S-Expressions genügten.

Dieser Instinkt ist geprüft, nicht nur behauptet: [`lib/meta-eval.my`](lib/meta-eval.my) ist ein metazirkulärer Evaluator — `eval`/`apply`, geschrieben in my-lisp selbst, dieselbe Beziehung, die McCarthys eigenes Paper von 1960 zu seinen Primitiven hatte — dispatcht an `car`/`cdr`/`cons`/`atom`/`eq`, statt sie neu zu implementieren. [`lib/unify.my`](lib/unify.my) und [`lib/reason.my`](lib/reason.my) sind eine kleine Unifikations- und Backward-Chaining-Inferenz-Engine im Geiste von McCarthys "Advice Taker"-Vorschlag von 1958 — symbolischer Mustervergleich, keine Statistik. Beide sind der Beweis, dass "eine kleine Sprache, die sich selbst wachsen lässt" eine arbeitende Aussage über diese Codebasis ist, kein Slogan.

### Advice Taker: beide Inferenzrichtungen, und beide Sprachrichtungen

[`lib/forward.my`](lib/forward.my) ist die andere klassische Hälfte symbolischer KI: eine CLIPS-artige Forward-Chaining-Engine (Working Memory, Ableitung bis zum Fixpunkt, `not`/`or`/`and`/`test`-Bedingungen), die sich mit `lib/reason.my` exakt dieselbe Regelform `(head cond1 cond2 ...)` teilt — dasselbe Regelliteral läuft unverändert rückwärts oder vorwärts. Beide Engines tragen echtes Truth Maintenance: JTMS mit einer und mit mehreren Begründungen, bei dem das Entfernen eines Fakts nur auf das kaskadiert, was wirklich von ihm abhing. [`lib/knowledge.my`](lib/knowledge.my) gruppiert Regeln in benannte, abfragbare Module; [`lib/clips-import.my`](lib/clips-import.my) liest echte CLIPS-`.clp`-Quelldateien von der Festplatte — `deffacts`, `defrule`, `deftemplate`s benannte Slots, `?x`-Variablen — und importiert sie als gewöhnliches my-lisp-Wissen, ein Beweis, dass alte symbolische KI-Systeme wiederverwendbar sind, nicht nur studierbar.

Erklärbarkeit zieht sich durch alles, nicht nachträglich angeflanscht: `explain-proof`/`reason-explain`/`provenance` verwandeln einen Beweisbaum in ein menschenlesbares "Warum", und [`lib/understand.my`](lib/understand.my)/[`lib/narrate.my`](lib/narrate.my) verbinden kontrollierte natürliche Sprache mit Wissensstrukturen in beide Richtungen — Text rein, Struktur raus, und zurück.

[`tests/fixtures/conformance.my`](tests/fixtures/conformance.my) und [`tests/fixtures/macro-conformance.my`](tests/fixtures/macro-conformance.my) existieren, weil Lisps Geschichte auch eine Geschichte auseinanderdriftender Dialekte ist — MacLisp, InterLisp, ein Dutzend Scheme-Varianten, Common Lisps Versuch, sie wiederzuvereinen. Diese Datei ist das eine, dem `fpga-lisp`s HDL-Lisp-Maschinen-Kern entsprechen muss, damit eine zweite Implementierung nie "nur ein weiterer Dialekt" wird.

Dieses Repository ist die kanonische Rust-Implementierung:

- [`crates/my-lisp`](crates/my-lisp) — der Kern: Parser, Evaluator, Umgebungen, exakte rationale Arithmetik.
- [`crates/my-lisp-cli`](crates/my-lisp-cli) — die `my-lisp`-Binärdatei (REPL + Dateiausführung).
- [`crates/my-lisp-wasm`](crates/my-lisp-wasm) — WebAssembly-Bindings für den Browser-REPL oben.
- [`crates/my-lisp-literate`](crates/my-lisp-literate) — Offset-Zuordnung von literate-Markdown-Quellcode.
- [`crates/my-lisp-lsp`](crates/my-lisp-lsp) — Language-Server-Protocol-Adapter über dem kanonischen Parser (Diagnostik, Dokumentsymbole, Hover, Go-to-Definition innerhalb einer Datei); auch als `my-lisp lsp` verfügbar.
- [`crates/my-lisp-host`](crates/my-lisp-host) — die OS-Fähigkeitsschicht (Dateisystem, Subprozesse, TCP), installiert im Capability-Register des Kerns nur, wenn ein Embedder dies bewusst aktiviert; der Kern selbst hat keinerlei OS-Zugriff.
- [`crates/my-lisp-semantic`](crates/my-lisp-semantic) — EXPERIMENTELL: eine Sanskrit-/Pāṇini-Semantikschicht (Transliteration, Semantic Atom Registry, die zwölf dhātu-Wurzeln, kāraka-Rollen), noch nicht an Parser/Evaluator angebunden.
- [`crates/wsm-guard-core`](crates/wsm-guard-core) — der eine gemeinsame Embed von `lib/core.my` + `lib/guard.wsm` und der Evaluate-and-Validate-Pfad, den jeder Guard-Verbraucher nutzt.
- [`crates/wsm-guard-slice`](crates/wsm-guard-slice) — ein minimaler ereignisgesteuerter Rust-Mechanismus, der ein begrenztes Ereignis pro Zeile rahmt und die Entscheidung der WSM-Policy überlässt.
- [`crates/wsm-guard-facts`](crates/wsm-guard-facts) — ein begrenzter, nur lesender Fakten-Adapter, der Git-/systemd-/Swarm-Beobachtungen in `(fact ...)`-Clauses für die Klassifikation durch die Guard-Policy normalisiert.
- [`crates/swarm-node`](crates/swarm-node) — eigenständiger Swarm-Koordinationsknoten.
- [`racket/`](racket/) — ein `#lang my-lisp`-Plugin für Racket/DrRacket mit dem Chez-Scheme-JIT.
- [`lib/core.my`](lib/core.my) — die gebootstrappte Standardbibliothek.
- [`lib/unify.my`](lib/unify.my) / [`lib/reason.my`](lib/reason.my) — Unifikation und Backward-Chaining-Inferenz.
- [`lib/forward.my`](lib/forward.my) — Forward-Chaining-Inferenz mit Truth Maintenance.
- [`lib/knowledge.my`](lib/knowledge.my) — benannte, abfragbare Wissensmodule.
- [`lib/world.my`](lib/world.my) — navigierbare und zweigvergleichbare unveränderliche Geschichte, lokales Schließen, atomare Aufnahme und Datenaustausch.
- [`lib/content-store.my`](lib/content-store.my) — ein unveränderlicher content-addressed Store, der gleiches Wissen und gleiche World-Geschichten dedupliziert.
- [`lib/understand.my`](lib/understand.my) / [`lib/narrate.my`](lib/narrate.my) — kontrollierte natürliche Sprache, beide Richtungen.
- [`lib/clips-import.my`](lib/clips-import.my) — importiert echte CLIPS-`.clp`-Quelldateien.

### Bauen und testen

```bash
cargo build --workspace
cargo test --workspace
```

### Racket / DrRacket-Unterstützung

Das Verzeichnis [`racket/`](racket/) enthält ein `#lang my-lisp`-Plugin für
Racket (Chez-Scheme-Backend). Es entspricht der Kernsemantik der
Rust-Implementierung: explizites `(quote x)` (Apostroph ist Teil eines
Symbols), exakte Dezimalliterale, Wahrheitswerte `t`/`()`, exakte Division
und die sieben McCarthy-Primitive. Lokale Installation:

```sh
raco pkg install --link --name my-lisp racket/
```

Danach lässt sich jede `.my`-Datei mit der Zeile `#lang my-lisp` in
DrRacket öffnen und mit Run ausführen, oder im Terminal mit
`racket datei.my`. Details stehen in
[`racket/README.md`](racket/README.md).

### Dokumentation

- [`docs/language-core.md`](docs/language-core.md) — der Sprachvertrag: Primitive, Bootstrap-Grenze, exakte Arithmetik.
- [`docs/FUNCTIONS.md`](docs/FUNCTIONS.md) — eine generierte Referenz jeder eingebauten und `lib/*.my`-Funktion, ein Abschnitt pro Bibliothek.
- [`docs/quote-tutorial.md`](docs/quote-tutorial.md) — ein Einsteiger-Rundgang durch Homoikonizität.
- [`docs/unify-tutorial.md`](docs/unify-tutorial.md) — ein kleines symbolisches KI-Beispiel: Unifikation, geschrieben in my-lisp selbst.
- [`docs/advice-ingestion.md`](docs/advice-ingestion.md) — die geschützte Wissensgrenze `understand → advise → reason → narrate`.
- [`docs/knowledge-package-format.md`](docs/knowledge-package-format.md) — versionierter reiner Datenaustausch für andere Projekte und KI-Adapter.
- [`docs/content-identity.md`](docs/content-identity.md) — kanonische Wissens-/Weltidentität vor kryptographischem Hashing.
- [`docs/canonical-serialization.md`](docs/canonical-serialization.md) — das portable `write-to-string`-Format und sein Round-Trip-Gesetz.
- [`docs/ecosystem-roadmap.md`](docs/ecosystem-roadmap.md) — der Vertragspfad durch my-lisp, den CML-Compiler und fpga-lisp.
- [`docs/clean-code.md`](docs/clean-code.md) / [`CLEAN_CODE_PLAN.md`](CLEAN_CODE_PLAN.md) — Clean-Code-Prinzipien und ihr ausführbarer Umsetzungsplan.
- [`docs/mccarthy-vision.md`](docs/mccarthy-vision.md) — wie John McCarthy selbst Ursprung und Entwicklung von Lisp beschrieb, von 1958 bis zu seinem Tod 2011, und wo dieses Projekt dieser Darstellung folgt oder von ihr abweicht.
- [`docs/testing.md`](docs/testing.md) — aktuelles Testinventar.
- [`docs/benchmarks.md`](docs/benchmarks.md) — Benchmark-Methodik und eine lokale Ausgangsmessung.
- [`docs/versioning.md`](docs/versioning.md) — warum die Versionshistorie dieses Repos so aussieht.

`my-lisp` begann innerhalb eines größeren IDE-Projekts, [`my-idea`](https://github.com/juv4uk/my-idea), und wurde hierher ausgelagert, um eigenständig zu bestehen. Eine zweite Implementierung derselben Sprache — ein von Grund auf neuer HDL-Kern für eine Lisp-Maschine, `fpga-lisp` — wird als separates, paralleles Repository entwickelt. Ein zuvor geplanter dritter Kern (ein C-Kern für Embedded-Ziele) wurde durch bewusste Entscheidung gestrichen; zwei unabhängige Implementierungen sind die aktuelle Verpflichtung.

Ein fünftes Schwester-Repository, [`my-lisp-panini`](https://github.com/juv4uk/my-lisp-panini), erforscht Pāṇinis Sanskrit-Grammatik (*Aṣṭādhyāyī*) als eigenständiges formales System und erzeugt das `panini-foundation`, auf dem die eigene Sanskrit-Semantik-Migration dieses Repos (`docs/sanskrit-semantic-migration.md`, die `SANSKRIT-P*`-Aufgaben) aufbaut. Es berührt `my-lisp` erst, wenn das eigene Machine-Model-Gate-Review abgeschlossen ist — das vollständige Forschungsmandat steht in der `AGENTS.md` jenes Repos.

## License · Ліцензія · Lizenz

[MIT](LICENSE)
