# Migration Specification: Sanskrit Semantic Atoms + Pāṇinian Model

SLP1 Canonical / IAST Presentation

Status: specification received 2026-08-12, Phase 0 (audit) in progress.
Not yet implemented — see "Обов'язковий звіт агента" (§39) for the
per-phase reporting format this migration follows, and §34 for the phase
list. Nothing in this document authorizes touching more than one phase in
a single commit.

## 0. Мета

Перевести базові семантичні атоми та назви фундаментальних операцій мови
на систему, засновану на санскритських dhātu (дієслівних коренях).

Канонічним машинним записом є SLP1.

IAST НЕ є внутрішнім представленням.
IAST використовується лише:

- в IDE;
- у документації;
- у підказках;
- у diagnostic messages для людини;
- у semantic inspector;
- у hover;
- за бажанням користувача як display mode.

Архітектурний принцип:

    meaning
       ↓
    semantic ID
       ↓
    SLP1
       ↓
   ┌───┴────┐
   ↓        ↓
 IAST    Devanāgarī
 display   display

Приклад:

    semantic ID: DHATU_DA
    SLP1:        dA
    IAST:        dā
    Devanāgarī:  दा
    class:       dhātu
    sense:       give / transfer

ВАЖЛИВО:

Не виконувати механічну заміну англійських назв функцій
санскритськими словами.

Це не localization/refactoring task.

Це створення semantic vocabulary layer.


## 1. Основні принципи

Дотримуватися таких правил:

1. SLP1 є canonical source representation.
2. SLP1 є ASCII-only.
3. IAST є presentation representation.
4. Unicode не повинен бути необхідним VM/FPGA/runtime.
5. Dhātu представляє базову дію/процес.
6. Kāraka представляє семантичну роль учасника дії.
7. Upasarga модифікує значення dhātu композиційно.
8. Морфологія не повинна бути частиною MVP.
9. Semantic ID має бути незалежним від орфографії.
10. Старий API не ламати до появи compatibility layer.


## 2. Не починати з перейменування коду

ЗАБОРОНЕНО починати з:

    add -> ...
    read -> ...
    write -> ...
    send -> ...

Спочатку провести аудит.

Знайти:

- builtins;
- primitives;
- special forms;
- VM instructions;
- parser symbols;
- compiler intrinsics;
- public functions;
- internal helper functions;
- UI commands;
- domain-specific functions.

Розділити їх на:

    LANGUAGE SEMANTICS
    IMPLEMENTATION
    UI
    INFRASTRUCTURE

Санскритизації підлягає передусім LANGUAGE SEMANTICS.

Наприклад:

    parse_ast()
    malloc()
    render_editor()
    websocket_connect()

не повинні автоматично отримувати санскритські назви.

Внутрішній Rust/C/Clojure/TypeScript код може залишатися англомовним.

Санскритська система належить мові, а не обов'язково реалізації компілятора.


## 3. Створити Semantic Atom Registry

Створити єдине authoritative джерело семантичних атомів.

Приблизна структура:

    semantic/
      atoms/
      dhatu/
      karaka/
      upasarga/
      transliteration/

Конкретний формат вибрати відповідно до архітектури репозиторію.

Кожен атом повинен мати щонайменше:

    id
    slp1
    iast
    devanagari
    category
    gloss
    semantic description
    aliases
    status

Приклад концептуально:

    {
      id: "DHATU_DA",
      slp1: "dA",
      iast: "dā",
      devanagari: "दा",
      category: "dhatu",
      gloss: "give",
      semantics: "transfer an entity from an agent toward a recipient"
    }

SLP1 string НЕ повинен використовуватися як єдиний semantic identity.

Правильно:

    DHATU_DA -> dA

Неправильно:

    semantic identity == "dA"

Це дозволить у майбутньому змінювати display/orthography,
не змінюючи AST, bytecode чи ABI.


## 4. Мінімальний Dhātu Core

Не імпортувати тисячі dhātu.

Створити маленьке експериментальне ядро.

Початкова кандидатна множина:

    kf     √kṛ     робити / створювати
    gam    √gam    рухатися / переходити
    dA     √dā     давати / передавати
    grah   √grah   брати / отримувати
    jYA    √jñā    знати
    dfS    √dṛś    бачити / спостерігати
    Sru    √śru    чути / приймати
    vac    √vac    говорити / повідомляти
    liK    √likh   писати
    paW    √paṭh   читати
    sTA    √sthā   стояти / перебувати
    BU     √bhū    бути / ставати

ВАЖЛИВО:

Перевірити SLP1/IAST/Devanāgarī форми за авторитетним
лексикографічним джерелом перед включенням у canonical registry.

Не покладатися на приблизну транслітерацію.

Для кожного dhātu задокументувати конкретний semantic sense,
який використовує мова.

Санскритський корінь може мати багато історичних значень.
Мова програмування повинна вибрати чітку operational semantics.


## 5. Kāraka Layer

Наступним шаром реалізувати невеликий набір Pāṇinian semantic roles.

Початково розглянути:

    kartf       kartṛ
    karman      karman
    karaRa      karaṇa
    sampradAna  sampradāna
    apAdAna     apādāna
    aDikaraRa   adhikaraṇa

Не прирівнювати kāraka механічно до grammatical case.

У semantic core kāraka — роль у відношенні/дії.

Приклад:

    (dA
      :kartf server
      :karman packet
      :sampradAna client)

IAST display:

    (dā
      :kartṛ server
      :karman packet
      :sampradāna client)

Семантика:

    server transfers packet to client

Ця структура повинна бути представлена AST,
а не зберігатися як декоративний текст.


## 6. AST

Не зберігати семантичні операції лише як raw strings.

Рекомендована концептуальна модель:

    SemanticCall {
        predicate: DHATU_DA,
        roles: {
            KARAKA_KARTR: server,
            KARAKA_KARMAN: packet,
            KARAKA_SAMPRADANA: client
        }
    }

Source:

    (dA
      :kartf server
      :karman packet
      :sampradAna client)

Parser:

    SLP1
      ↓
    symbol resolution
      ↓
    semantic IDs
      ↓
    AST

Таким чином після parsing SLP1 перестає бути головним носієм значення.


## 7. Display Pipeline

Реалізувати незалежний display/transliteration layer.

Має підтримувати принаймні:

    canonical
    iast

Бажано архітектурно передбачити:

    devanagari

API концептуально:

    render_atom(atom_id, DisplayMode::SLP1)
    render_atom(atom_id, DisplayMode::IAST)
    render_atom(atom_id, DisplayMode::Devanagari)

Результат:

    DHATU_DA

    SLP1       -> dA
    IAST       -> dā
    Devanagari -> दा

Ніяка зміна display mode не повинна змінювати AST.


## 8. IDE

IDE повинна розрізняти:

    storage representation
    semantic representation
    visual representation

За замовчуванням файл зберігається в SLP1.

Наприклад фізичний source:

    (dA
      :kartf server
      :karman packet
      :sampradAna client)

IDE може показувати режим IAST:

    (dā
      :kartṛ server
      :karman packet
      :sampradāna client)

Але це НЕ повинно непомітно переписувати файл Unicode-символами.


## 9. Hover / Semantic Inspector

Для semantic atom IDE повинна вміти показувати:

    dA
    ─────────────────

    IAST: dā
    Devanāgarī: दा

    Type:
    dhātu

    Root:
    √dā

    Core semantic:
    give / transfer

    Roles:
    kartṛ
    karman
    sampradāna

За можливості додати посилання на linguistic metadata.


## 10. Input

Canonical input:

    SLP1

Пізніше можна підтримати input adapters:

    IAST
    Devanāgarī
    ITRANS
    Harvard-Kyoto

але вони повинні нормалізуватися:

    IAST ───────┐
    Devanāgarī ─┤
    ITRANS ─────┤
    HK ─────────┤
                ↓
              SLP1
                ↓
           semantic ID

Не створювати окремі semantic atoms для різних транслітерацій.


## 11. Upasarga

НЕ реалізовувати великий upasarga system у першому commit.

Але architecture повинна дозволяти:

    modifier + dhātu -> derived semantic operation

Концептуально:

    upasarga
       +
    dhātu
       ↓
    derived predicate

Не створювати сотні unrelated builtins, якщо операція може бути
виражена композицією.

Derived operation повинна зберігати інформацію про походження:

    DerivedPredicate {
        base: DHATU_GAM,
        prefix: ...
    }


## 12. Pāṇinian Grammar

Не реалізовувати всю Aṣṭādhyāyī.

MVP використовує Паніні як архітектурне натхнення для:

    dhātu
    kāraka
    compositional derivation
    semantic roles

Не реалізовувати поки:

- повну sandhi;
- повну nominal morphology;
- повну verbal morphology;
- всі lakāra;
- повну derivational grammar;
- natural-language Sanskrit parser.

Це окремі майбутні шари.


## 13. Старі функції

Заборонено одним commit видалити старі англійські builtins.

Спочатку створити aliases/compatibility mapping.

Наприклад концептуально:

    give -> DHATU_DA
    transfer -> DHATU_DA

або:

    legacy_builtin("send")
        ↓
    semantic lowering
        ↓
    DHATU_DA

Після цього:

    old syntax
       ↓
    semantic IR
       ↑
    SLP1 syntax

можуть співіснувати.


## 14. Semantic IR

Ввести чітку межу:

    source language
          ↓
        parser
          ↓
    semantic AST
          ↓
    semantic IR
          ↓
       low IR
          ↓
       bytecode
          ↓
        VM/FPGA

SLP1 не повинен просочуватися до ISA без необхідності.

FPGA не повинен знати:

    dA
    dā
    दा

FPGA має бачити щось на кшталт:

    OPCODE_TRANSFER

або numeric opcode.

Наприклад:

    (dA ...)
        ↓
    DHATU_DA
        ↓
    TRANSFER IR
        ↓
    opcode 0x17

Це принципово важливо.


## 15. Semantic ISA

Розглянути semantic atoms як високорівневий Semantic ISA.

Тобто:

    Sanskrit/Pāṇinian layer
              ↓
         Semantic ISA
              ↓
              IR
              ↓
        Machine ISA

Не змішувати ці два ISA.

Semantic ISA описує значення.

Machine ISA описує виконання.


## 16. Functions vs Actions

Не намагатися перетворити кожну function на dhātu.

Наприклад:

    sin
    cos
    sqrt
    map
    vector
    integer
    markdown

можуть належати іншим semantic categories.

Створити ontology/category system:

    dhatu
    karaka
    entity
    property
    relation
    mathematical
    structural
    special-form
    literal
    type

Санскритська модель не означає, що все має бути дієсловом.


## 17. Special Forms

Окремо проаналізувати:

    if
    let
    lambda
    quote
    define
    begin
    match

Не давати їм випадкові Sanskrit translations.

Спочатку визначити їхню semantic category.

Наприклад lambda може бути language constructor,
а не dhātu.

Special forms повинні мігрувати лише після окремого semantic design.


## 18. Exact Semantics

Для кожного нового atom створити specification.

Приклад:

    ID:
    DHATU_DA

    Canonical:
    dA

    IAST:
    dā

    Category:
    dhātu

    Operational meaning:
    transfer ownership/reference/value/information
    from kartṛ toward sampradāna.

    Required roles:
    karman

    Optional roles:
    kartṛ
    sampradāna

    Effects:
    context dependent

    Purity:
    context dependent

Це важливіше за красиву назву.


## 19. Type System

Не прив'язувати dhātu напряму до одного concrete type.

Наприклад dA потенційно може описувати:

    value transfer
    message transfer
    file transfer
    ownership transfer
    capability transfer

Конкретна реалізація визначається:

    semantic predicate
        +
    roles
        +
    argument types
        +
    context

Це відкриває шлях до multimethod / dispatch semantics.


## 20. Parser Validation

Parser повинен перевіряти canonical SLP1 atoms.

Потрібні тести:

    valid SLP1
    invalid SLP1
    unknown atom
    known dhātu
    known kāraka
    duplicate role
    missing required role
    unsupported role

Не дозволяти silent fallback:

    unknown SLP1 -> arbitrary symbol with semantic meaning

Звичайні user symbols можуть існувати,
але semantic atoms повинні мати окремий namespace/category.


## 21. Namespaces

Розглянути явні namespaces.

Наприклад внутрішньо:

    dhatu/dA
    karaka/kartf

Source syntax може залишатися коротким:

    dA
    :kartf

Resolver повинен знати category.


## 22. Transliteration Tests

Обов'язково створити round-trip tests.

Наприклад:

    SLP1 -> IAST -> SLP1

має повертати canonical representation.

Перевірити:

    dA
    kf
    jYA
    dfS
    Sru

та всі kāraka atoms.

Додати окремі edge cases для:

    vocalic r
    retroflex consonants
    palatals
    aspirates
    anusvāra
    visarga
    long vowels

Не створювати transliteration table "по пам'яті".
Вона повинна бути перевірена.


## 23. Documentation

Для кожного semantic atom документація показує три рівні:

    SLP1: dA
    IAST: dā
    Devanāgarī: दा

та окремо:

    semantic meaning

Не використовувати англійський gloss як definition.

Наприклад:

    gloss: give

лише допомагає людині.

Canonical semantics має бути формально описана окремо.


## 24. Source Code Examples

Документація повинна вважати SLP1 canonical source.

Наприклад:

    (dA
      :kartf server
      :karman packet
      :sampradAna client)

Поруч можна показати presentation:

    IAST display:

    (dā
      :kartṛ server
      :karman packet
      :sampradāna client)

Не змішувати SLP1 та IAST в одному canonical example.


## 25. Git / Storage

У canonical source бажано ASCII.

Переваги:

- простий Git diff;
- прості terminals;
- FPGA tooling;
- embedded tooling;
- ASCII parsers;
- stable identifiers;
- менше Unicode normalization issues.

IAST Unicode не повинен випадково потрапляти у canonical files,
якщо файл не є документацією.


## 26. Unicode Normalization

IAST presentation layer повинен мати explicit Unicode policy.

Рекомендовано NFC для display/export.

Але semantic equality:

    dā == dā

не повинна визначатися raw Unicode byte comparison.

Спочатку transliteration/normalization,
потім semantic ID resolution.


## 27. Performance

Не виконувати повну transliteration при кожному VM instruction.

Transliteration належить:

    parser/input
    IDE
    diagnostics
    documentation

Runtime працює з:

    semantic IDs
    enums
    interned atoms
    numeric IDs

Не зі строками.


## 28. Serialization

Якщо AST/IR серіалізується, визначити canonical representation.

Рекомендація:

    semantic ID + canonical SLP1 metadata/version

Не серіалізувати лише IAST.

Додати vocabulary/schema version:

    semantic-vocabulary: 0.1

Це дозволить змінювати словник контрольовано.


## 29. Vocabulary Versioning

Semantic vocabulary повинен версіонуватися.

Наприклад:

    semantic-core v0.1

Зміна:

    spelling
    semantics
    role requirements
    derivation

може бути breaking change.

Не змінювати значення існуючого atom мовчки.


## 30. Linter

У перспективі додати semantic linter.

Він має ловити:

    unknown dhātu
    invalid kāraka
    impossible role combination
    deprecated atom
    noncanonical spelling
    IAST used where SLP1 is required

Приклад:

    dā

у canonical source:

    warning:
    IAST atom detected.
    Canonical SLP1 spelling is:

        dA


## 31. IDE Autocomplete

Autocomplete повинен шукати одночасно за:

    SLP1
    IAST
    English/Ukrainian gloss
    semantic category

Наприклад користувач вводить:

    give

IDE пропонує:

    dA

    √dā
    give / transfer
    dhātu

Але вставляє у canonical source:

    dA


## 32. IDE Search

Semantic search повинен у перспективі дозволяти:

    search: dA
    search: dā
    search: give
    search: передати

і знаходити той самий:

    DHATU_DA

Це одна з причин не використовувати spelling як semantic identity.


## 33. Mermaid / Markdown / LaTeX

Не санскритизувати зовнішні стандарти.

Назви:

    Markdown
    Mermaid
    LaTeX

залишаються такими, якими їх визначає зовнішня екосистема.

Semantic layer може описувати операції над ними,
але не перейменовувати самі стандарти.


## 34. Migration Strategy

Виконувати поетапно.

PHASE 0
Audit.

PHASE 1
Transliteration library/tests.

PHASE 2
Semantic Atom Registry.

PHASE 3
5-12 experimental dhātu.

PHASE 4
Kāraka model.

PHASE 5
AST semantic IDs.

PHASE 6
Compatibility aliases.

PHASE 7
IDE hover/autocomplete.

PHASE 8
IAST display mode.

PHASE 9
Semantic IR lowering.

PHASE 10
Upasarga experiments.

Не виконувати всі фази одним commit.


## 35. Перший експеримент

Перший vertical slice повинен бути МАЛИМ.

Рекомендовано реалізувати один dhātu:

    dA / dā

і три ролі:

    kartf
    karman
    sampradAna

Canonical source:

    (dA
      :kartf server
      :karman packet
      :sampradAna client)

Потрібно пройти весь pipeline:

    source
      ↓
    tokenizer
      ↓
    parser
      ↓
    atom resolver
      ↓
    semantic AST
      ↓
    semantic IR
      ↓
    existing runtime operation

IDE:

    dA -> dā -> दा

Якщо цей один приклад проходить end-to-end,
архітектура придатна до розширення.


## 36. Другий експеримент

Після dA реалізувати:

    gam

Перевірити, чи модель може описувати:

    actor
    source
    destination/location

Не створювати спеціальну архітектуру тільки під dA.

Якщо gam не вкладається у модель,
переробити abstraction до розширення vocabulary.


## 37. Заборонені дії

НЕ:

- перейменовувати всі функції одразу;
- перекладати English -> Sanskrit словником;
- використовувати IAST як internal identifier;
- використовувати Devanāgarī як VM identifier;
- прив'язувати bytecode до Unicode;
- реалізовувати всю граматику Паніні;
- імпортувати тисячі dhātu;
- змішувати linguistic semantics та machine ISA;
- ламати legacy syntax;
- змінювати runtime без необхідності;
- змінювати UI одночасно з semantic core;
- вигадувати власну Sanskrit transliteration;
- вважати English gloss формальним значенням;
- робити Sanskrit декоративним naming convention.


## 38. Definition of Done для першої версії

semantic-core v0.1 вважається готовим, коли:

**Progress summary (`DOCS-SANSKRIT-SPEC-PROGRESS-SUMMARY`, updated as phases
land — see commit citations for the ground truth, not this table):**

| # | Checkbox | Status | Evidence |
|---|---|---|---|
| 1 | canonical SLP1 policy визначено | ✅ done | `transliteration.rs` module doc; spec §1 |
| 2 | SLP1 validator існує | 🟡 partial | `slp1_to_devanagari`/`slp1_to_iast` reject unknown chars (functional validation via error), no standalone "just validate" API |
| 3 | SLP1 -> IAST converter існує | ✅ done | `transliteration.rs`, commit `23aecec` |
| 4 | IAST -> SLP1 converter існує | ✅ done | `transliteration.rs`, commit `23aecec` |
| 5 | round-trip tests проходять | ✅ done | 9 tests in `transliteration.rs`, 8 in `devanagari.rs` |
| 6 | Semantic Atom Registry створено | ✅ done | `atoms.rs`, commit `f7cfba3` |
| 7 | semantic atoms мають stable IDs | ✅ done | `id ≠ slp1` enforced by test, commit `f7cfba3` |
| 8 | мінімальний dhātu vocabulary реалізовано | ✅ done | 12 dhātu, `atoms.rs`, commit `7a115d2` |
| 9 | мінімальний kāraka vocabulary реалізовано | ✅ done | 6 kāraka + `SemanticCall`, commit `bd24f79` |
| 10 | parser resolve-ить SLP1 до semantic ID | ❌ not done | design proposed (`docs/sanskrit-p5-parser-design.md`, commit `278ffa0`), not implemented — `SANSKRIT-P5-AST-SEMANTIC-IDS` |
| 11 | AST не залежить від IAST | 🟡 vacuously true | `SemanticCall` itself has no IAST dependency, but nothing yet *produces* one from real parsed source (depends on #10) |
| 12 | runtime не залежить від IAST/Devanāgarī | 🟡 vacuously true | `eval/mod.rs` untouched by the migration so far — true because nothing's wired in yet, not yet a tested guarantee |
| 13 | IDE може показати IAST | ❌ not done | `SANSKRIT-P7-IDE-HOVER` not started |
| 14 | hover показує linguistic + semantic metadata | ❌ not done | `SANSKRIT-P7-IDE-HOVER` not started |
| 15 | compatibility layer підтримує старий код | ❌ not done | mapping enumerated (`docs/sanskrit-p6-alias-enumeration.md`, commit `fefeaae`: only 4 of 15 builtins map cleanly), `alias_table` itself not implemented |
| 16 | документація містить SLP1 + IAST | ✅ done | every atom entry in `atoms.rs`; this spec's own worked examples |
| 17 | semantic vocabulary має version | ❌ not done | no `semantic-core v0.1`-style version tag exists anywhere in code or the registry — a real gap, not yet even tracked as its own task |
| 18 | існують end-to-end tests | 🟡 partial | extensive component-level tests (43 across the `semantic` module as of `bd24f79`) including both spec worked examples (§0/§35, §36) built directly; no true source-text→parse→resolve→eval pipeline test yet, since #10 isn't done |

**Tally: 9 done, 4 partial/vacuous, 5 not done**, out of 18. Phases 0-4
complete; P5 has a design (not code); P6 has an enumeration (not code);
P7-P10 not started. The vocabulary-versioning gap (#17) surfaced by this
summary is new — not previously tracked as its own task.

- [ ] визначено canonical SLP1 policy
- [ ] існує SLP1 validator
- [ ] існує SLP1 -> IAST converter
- [ ] існує IAST -> SLP1 converter
- [ ] round-trip tests проходять
- [ ] створено Semantic Atom Registry
- [ ] semantic atoms мають stable IDs
- [ ] реалізовано мінімальний dhātu vocabulary
- [ ] реалізовано мінімальний kāraka vocabulary
- [ ] parser resolve-ить SLP1 до semantic ID
- [ ] AST не залежить від IAST
- [ ] runtime не залежить від IAST/Devanāgarī
- [ ] IDE може показати IAST
- [ ] hover показує linguistic + semantic metadata
- [ ] compatibility layer підтримує старий код
- [ ] документація містить SLP1 + IAST
- [ ] semantic vocabulary має version
- [ ] існують end-to-end tests


## 39. Обов'язковий звіт агента

Після кожної фази надати:

    Phase:
    COMPLETE / PARTIAL / BLOCKED

    Files changed:

    Semantic atoms added:

    Canonical SLP1 forms:

    IAST mappings:

    AST changes:

    Parser changes:

    Runtime changes:

    IDE changes:

    Compatibility impact:

    Tests:
    [PASS/FAIL]

    Existing tests:
    [PASS/FAIL]

    Breaking changes:
    NONE / list

    Questions requiring design decision:

    Next recommended phase:


## 40. Головний архітектурний принцип

Не будувати:

    Sanskrit-looking programming language

Будувати:

    Sanskrit/Pāṇinian semantic model
                ↓
          canonical SLP1
                ↓
          Semantic Atoms
                ↓
           Semantic AST
                ↓
           Semantic IR
                ↓
        existing low-level IR
                ↓
             VM/FPGA

IAST, Devanāgarī, English та Ukrainian є різними
людськими представленнями одного semantic object.

У кінцевому підсумку:

    dA
    dā
    दा
    give
    давати

можуть допомагати людині знайти один об'єкт:

    DHATU_DA

але тільки semantic ID визначає його машинну тотожність.
