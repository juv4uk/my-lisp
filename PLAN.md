# План: виділення Rust+Lisp в окремий репозиторій

Робочий план для нової чат-сесії Claude Code, запущеної в `C:\Users\juv4u\Documents\my-lisp` (git worktree гілки `my-lisp`, вже запушений як публічний [github.com/juv4uk/my-lisp](https://github.com/juv4uk/my-lisp)). Прочитай спершу `CLAUDE.md` — там принципи мови й підтверджені рішення. Цей файл — конкретні кроки, не філософія.

**Поточний стан (оновлено):** Кроки 1–7 фактично вже виконані — репозиторій `my-lisp` виділений через `git filter-repo`, `Cargo.toml` звужений до 4 крейтів (`my-lisp`, `-cli`, `-wasm`, `-literate`), `src-tauri`/`src-cljs`/CLJS-тулінг відсутні, README/versioning.md переписані під мову, реліз-теги свої (`l*`-префікс). Не зроблено: Крок 8 — `github.com/juv4uk/my-idea` досі містить власну копію крейтів my-lisp і **не посилається** на цей виділений репозиторій як на залежність (перевірено на GitHub 2026-08-08).

## Крок 0 — узгодити з користувачем перед стартом

Не виконувати нижче нічого без явного підтвердження користувача на кожен великий крок (видалення файлів, переписування історії, публічний push). План — не мандат діяти автономно.

## Крок 1 — інструментарій для чистої історії

- Перевірити `git filter-repo --version`. Якщо відсутній — треба Python 3 (зараз недоступний у PATH, `python3`/`python` не знайдено) + `pip install git-filter-repo`, або `scoop install git-filter-repo` (у користувача вже є scoop у PATH).
- Без `git-filter-repo` можна зробити грубіше: залишити тільки потрібні файли одним великим комітом (втратити per-file історію для решти), або спробувати `git filter-branch` (повільний, вбудований, але офіційно deprecated).
- **Рекомендація:** дочекатись `git-filter-repo`, не робити "грубе" видалення, якщо користувач хоче зберегти історію crates/my-lisp.

## Крок 2 — визначити periметр (що лишається)

Ймовірний список того, що належить Rust+Lisp репо:
- `crates/my-lisp/` — ядро
- `crates/my-lisp-cli/` — CLI/REPL бінарник
- `crates/my-lisp-wasm/` — WASM-біндінги
- `crates/my-lisp-literate/` — literate-Markdown підтримка
- `lib/core.my` — bootstrap-бібліотека
- `Cargo.toml` / `Cargo.lock` (workspace root) — **потребує переписування**: зараз описує весь my-idea workspace (включно з `src-tauri`), треба звузити до чотирьох my-lisp-крейтів
- `docs/language-core.md`, `docs/quote-tutorial.md`, `docs/testing.md` (тільки Rust-частина), `docs/versioning.md` (адаптувати — версіонування тут своє, не успадковане від my-ide)
- `crates/my-lisp/tests/fixtures/conformance.json` (якщо є — перевірити шлях) і сам `crates/my-lisp/tests/mccarthy.rs`
- `benchmarks/*.my` + `scripts/benchmark.mjs` + `crates/my-lisp/examples/benchmark.rs` (якщо бенчмарки залишаються тут)
- `public/my-lisp-cli-web.html` + `scripts/make-portable-web.mjs` (якщо хочемо тримати веб-REPL демо в цьому репо, а не в my-idea)
- `LICENSE` (MIT, скопіювати як є)
- Корінний `README.md` — **переписати з нуля** під мову, не під IDE (поточний README — про my-idea)
- `CLAUDE.md` — вже є, оновлювати по ходу
- Новий/адаптований `.gitignore` (звузити з my-idea-версії — прибрати Android/Tauri/CLJS-специфічні рядки)

**Явно НЕ лишається:** `src-cljs/`, `src-tauri/`, `public/` (крім вищезгаданого HTML), `package.json`/`package-lock.json`/npm-тулінг, `shadow-cljs.edn`, `app-icon.svg`, `.github/workflows/*` (крім, можливо, адаптованого CI для Rust-тестів), `docs/android-release.md`, `docs/platform-roadmap.md`, `docs/release-assets.md`, `docs/windows-arm64.md`, `docs/source-files.md` (IDE-специфічний), `docs/benchmarks.md` (якщо бенчмарки не переносяться), `scripts/release.ps1`/`release.sh` (IDE-релізний процес), `scripts/setup-android-signing.ps1`, `scripts/configure-android-signing.mjs`, `scripts/verify-windows-architecture.ps1`, `test_parse.rs` (старий debug-скрипт для перевірки `pulldown_cmark`-парсингу literate-блоків, закомічений у `8c16ea1`; не частина жодного крейта — ймовірно можна викинути, а не переносити).

**Обговорити з користувачем окремо:** чи `crates/my-lisp-wasm` і веб-REPL демо лишаються тут, чи в my-idea (там вони теж використовуються для Language Lab — можливе дублювання коду між двома репо, якщо не виділити спільний крейт).

## Крок 3 — виконати вирізання історії

Коли периметр узгоджено і `git-filter-repo` встановлено:

```bash
git filter-repo --path crates/my-lisp --path crates/my-lisp-cli \
  --path crates/my-lisp-wasm --path crates/my-lisp-literate \
  --path lib/core.my --path LICENSE \
  --path docs/language-core.md --path docs/quote-tutorial.md \
  --path docs/testing.md --path CLAUDE.md \
  ... # інші шляхи з Кроку 2
```

Робити це **в самій `C:\Users\juv4u\Documents\my-lisp` папці** (вона вже ізольований worktree/клон гілки, не основний `my-idea`), щоб не зачепити `my-idea`. `git-filter-repo` за замовчуванням переписує весь локальний репозиторій, тому перед запуском варто ще раз перевірити, що `git remote -v` тут показує тільки `my-lisp`/`origin` на `github.com/juv4uk/my-lisp`, а не залежить від `my-idea`.

## Крок 4 — новий Cargo workspace root

Переписати корінний `Cargo.toml`:
```toml
[workspace]
members = ["crates/my-lisp", "crates/my-lisp-cli", "crates/my-lisp-wasm", "crates/my-lisp-literate"]
resolver = "2"
```
Прибрати `src-tauri` з членів workspace. Перевірити, чи є root-level release-профіль (`opt-level = "z"`, `lto = true` — згадувалось у `crates/my-lisp-wasm/Cargo.toml` як "визначено в корені workspace") — перенести його сюди без змін.

## Крок 5 — новий README

Написати з нуля, трилінгвально (EN/UA/DE, конвенція проєкту), на основі змісту `crates/my-lisp/README.md` і `docs/language-core.md`, але як головний вхідний документ репо:
- "A small language that grows itself"
- Швидкий старт: `cargo run -p my-lisp-cli`, або скачати `my-lisp-cli-web.html` без встановлення
- Посилання на `docs/quote-tutorial.md`
- McCarthy-контракт, точна раціональна арифметика — коротко, з посиланням на `docs/language-core.md`
- Тести: `cargo test --workspace`

## Крок 6 — CI

Новий/адаптований `.github/workflows/ci.yml`: `cargo test` для чотирьох крейтів (як зараз у my-idea, тільки без npm/CLJS кроків). Окремо подумати, чи потрібен release-workflow тут (публікація `my-lisp-cli` бінарників для кількох ОС — уже є робочий приклад у `my-idea`'s `cli-release.yml`, можна адаптувати).

## Крок 7 — версіонування

`crates/my-lisp` вже має власну версію `0.1.0` (незалежну від my-idea `0.x.x`, за `docs/versioning.md`). Вирішити з користувачем: чи цей репо стартує тегами з `v0.1.0`, чи з чогось іншого — не вигадувати самому.

## Крок 8 — зв'язок з my-idea

Після того як новий репо стабілізується, повернутись до `my-idea` і вирішити (окреме обговорення з користувачем, не автоматично):
- чи `my-idea` тепер тягне my-lisp як git submodule / залежність з crates.io / просто копіює на реліз,
- чи `crates/my-lisp*` в `my-idea` видаляються повністю,
- як синхронізувати виправлення (баг в evaluator має чинитись в одному місці, не в двох копіях).

## Крок 9 — напрямки розвитку мови, узгоджені з духом Маккарті

Не мандат діяти автономно — як і решта плану, кожен пункт узгоджувати з користувачем перед стартом. Пріоритет — зверху вниз.

1. ✅ **Метациркулярний evaluator самою my-lisp.** Зроблено: [`lib/meta-eval.my`](../lib/meta-eval.my) (окремий файл, не частина завжди завантажуваного `lib/core.my`), 9 тестів у `crates/my-lisp/tests/meta_eval.rs`, короткий приклад у `docs/quote-tutorial.md` (Етап 6). `(my-eval expr env)` інтерпретує quoted-списки через `read`/`car`/`cdr`/`cond`/`lambda`/`eq`/`atom`, за схемою eval/apply Маккарті 1960 року — примітиви диспетчеризуються напряму, не переписані заново.
2. ✅ **`tests/fixtures/conformance.json` — тримати як єдину точку правди**, коли з'явиться C-ядро чи HDL-реалізація (див. CLAUDE.md, "Confirmed future direction"). Зроблено: [`tests/fixtures/README.md`](../tests/fixtures/README.md) — формат, правила (спільна сесія між фікстурами, незмінність опублікованих значень), і новий тип фікстур — `{ "expr": "...", "error": "ErrorKind" }` для очікуваних помилок, не лише успішних обчислень (раніше контракт не перевіряв взагалі, чи падає щось правильним видом помилки). Не давати нових реалізацій ставати "ще одним несумісним діалектом" — саме ця фрагментація Lisp-екосистеми 70-80х засмучувала Маккарті найбільше.
3. ✅ **Невеликий символьний AI-приклад: машина логічного висновування (Advice Taker).** Зроблено: [`lib/unify.my`](../lib/unify.my) — unification (з повноцінним occurs-check) та [`lib/reason.my`](../lib/reason.my) — backward-chaining Prolog-подібний рушій (зі standardizing apart для підтримки рекурсивних правил). Логічні змінні представлені як `(var name)`-пари, а не голі символи (`eq` вимагає атомів, але ми навчили уніфікатор працювати з `equal?` для імен). Тести у `crates/my-lisp/tests/unify.rs` та `crates/my-lisp/tests/reason.rs`. Це наблизило my-lisp до первісного бачення Джона Маккарті (програмування як навчання системи фактам).
4. ✅ **Пояснюваність (explainability).** Зроблено: `explain-proof` у [`lib/reason.my`](../lib/reason.my) будує дерево доведення (`proved`/`proved-not` вузли) і друкує людський трейс — "чому" система дійшла висновку, не лише "що". Тест `test_explain_proof` у `crates/my-lisp/tests/reason.rs`.
5. ✅ **Модульність знань (knowledge packages).** Зроблено: [`lib/knowledge.my`](../lib/knowledge.my) — `defmodule`, ізольовані запити через `reason-in`, і `tell-knowledge` з перевіркою конфліктів (доводить заперечення нового факту перед додаванням). Відповідає пункту "знання — пакети, які можна завантажувати, перевіряти, комбінувати" з `private/lisp-to-knowledge.md` §10.
6. ❌ **NLP-міст (`understand`), перша версія — видалено.** Була зроблена (`crates/my-lisp-cli/src/llm.rs` + `:tell`/`:ask`), але лишалась REPL-хаком: єдине місце в кодбазі без тестів, без trilingual-конвенції, з мережевою залежністю та зовнішнім API-ключем — не на рівні строгості решти проєкту. Видалено разом з `reqwest`/`serde`/`serde_json` з `crates/my-lisp-cli/Cargo.toml`. Якщо повертатись до цього кроку (трійка `eval`/`reason`/`understand` з `private/lisp-to-knowledge.md` §9) — робити одразу з тестами й документацією, а не потім.
7. ✅ **Провенанс тверджень (statement provenance) — перший крок.** Зроблено: `reason-explain` у [`lib/reason.my`](../lib/reason.my) явно розрізняє "доведено" (пояснює перше доведення через `explain-proof`) від "не можу довести" (прямо каже про це замість мовчазного порожнього списку). Тести: `reason_explain_explains_a_provable_goal`, `reason_explain_says_so_when_a_goal_cannot_be_proved` у `crates/my-lisp/tests/reason.rs`. Проте-дерево вже саме по собі несе джерело (факт vs правило — видно з порожнього/непорожнього списку підцілей у вузлі `proved`), тож структуроване `(джерело ...) (впевненість ...)` з §12 залишається можливим наступним кроком, якщо знадобиться — поки що не потрібне.
8. ✅ **Атом як вхід у поняття (concept entry point).** Зроблено: `describe` у [`lib/knowledge.my`](../lib/knowledge.my) приймає символ і назву модуля, повертає всі факти цього модуля, що згадують символ (наприклад, `(describe 'earth 'astronomy)` → `((planet earth))`). Без нового примітиву — лягло на вже наявну модель фактів `defmodule`, лише новий напрям запиту (від символу назовні, а не від цілі вглиб, як `reason`). Тести: `test_describe_collects_every_fact_about_a_symbol`, `test_describe_unknown_module`, `test_describe_symbol_with_no_facts` у `crates/my-lisp/tests/knowledge.rs`.
9. ✅ **Лічильник використання (usage counting) — вимірювана "живість" дерева знань.** Зроблено: `count-usage`/`merge-usage` у [`lib/reason.my`](../lib/reason.my) обходять дерево доведення (`proved`-вузли) і повертають `(голова-правила . скільки-разів-використано)`. Справжня мутація (`(times-used N)` inline у факті, інкрементована всередині `prove-rule`) виявилась неможливою без нового `set!`-примітиву: `def` в my-lisp мутує лише поточний фрейм ([environment.rs:70](../crates/my-lisp/src/environment.rs:70)), а `prove-rule` виконується у вкладених `lambda`-викликах — тож обрано чисто функціональний підхід (рахувати з уже наявного дерева доведення) замість додавання мутабельного стану в рушій. Накопичення між запитами — `*usage-counts*`/`record-usage!`/`usage-of` у [`lib/knowledge.my`](../lib/knowledge.my), за тим самим top-level `def`-паттерном, що й `*knowledge-base*` (`record-usage!` **має** викликатись на верхньому рівні, не всередині `let` — інакше мовчки створює локальний binding, який зникає). Побічно виявлено й задокументовано баг reader'а: dotted-pair-літерали (`'(p . 0)`) не парсяться як справжні dotted pairs (`.` читається як звичайний символ), хоча printer саме так друкує реальні dotted pairs — див. `docs/testing.md`. Тести: `count_usage_counts_each_rule_head_that_contributed_to_a_proof`, `count_usage_sums_repeated_use_of_the_same_fact` у `crates/my-lisp/tests/reason.rs`; `test_record_usage_accumulates_across_separate_queries`, `test_usage_of_unrecorded_rule_is_zero` у `crates/my-lisp/tests/knowledge.rs`.
10. ✅ **Провенанс тверджень — повна структура.** Зроблено: `provenance`/`provenance-list` у [`lib/reason.my`](../lib/reason.my) перетворюють вузол дерева доведення на явний запис `(statement ціль (source fact|rule) (rule голова-правила) (derived-from список-підтверджень))`, рекурсивно для всього дерева. Свідомо без полів `(confidence ...)`/`(time ...)` з `private/lisp-to-knowledge.md` §12 — рушій точний і детермінований (той самий принцип точних чисел, що й в арифметиці), тож числова "впевненість" була б вигаданою точністю. Чиста надбудова над вже наявним деревом доведення (той самий підхід, що й `count-usage`), без нового примітиву. Тести: `provenance_marks_a_bare_fact_as_source_fact_with_no_derivation`, `provenance_marks_a_rule_application_as_source_rule_with_its_derivation` у `crates/my-lisp/tests/reason.rs`.
11. ✅ **NLP-міст, контрольована природна мова (без LLM).** Зроблено: [`lib/understand.my`](../lib/understand.my) — `understand` зіставляє фіксовані форми **списку слів** (не рядка — у my-lisp нема рядкових примітивів: split, доступ до символів; додавати їх лише заради токенізації означало б розширювати поверхню Rust built-in) зі знаннєвим clause: `(X is a Y)`/`(X is Y)` → факт `((Y X))`, `(X V Y)` → факт `((V X Y))`, `(all X have Y)` → правило `((has (var w) Y) (X (var w)))`. Без морфології (без відкидання множини, без відмінювання) — виклик уже подає точну однину класу. Результат напряму згодовується в `reason` без ручного редагування. Це перша половина мосту "текст → структура" з `private/lisp-to-knowledge.md` §6 — повний LLM-гібрид (друга половина, вільний текст) залишається окремим майбутнім кроком, і має відповідати тій самій строгості (тести, trilingual), що й тут. Тести: `understand_is_a_produces_a_class_membership_fact`, `understand_is_without_article_produces_the_same_fact`, `understand_subject_verb_object_produces_a_relation_fact`, `understand_all_have_produces_a_universal_rule`, `understand_output_is_usable_directly_as_a_reason_rule` у `crates/my-lisp/tests/understand.rs`.

12. 🔶 **Forward-chaining рушій у стилі CLIPS — Крок 1-4 з N.** Крок 1 зроблено: [`lib/forward.my`](../lib/forward.my) — `*working-memory*` (той самий top-level `def`-паттерн, що й `*knowledge-base*`/`*usage-counts*`), `assert-fact!` (макрос, той самий трюк, що й `record-usage!`/`tell-knowledge`), `fire-rule` — унікація шаблону одного правила з одним фактом і підстановка в template через `apply-subst`, або `'no-match` при невдачі. Крок 2 зроблено: `fire-rule-on-facts` — застосування одного правила до всього списку фактів, збір нових фактів, `'no-match` мовчки відкидається (guard `(atom result)` перед `eq`, той самий паттерн, що й `var?` у `lib/unify.my` — template може бути складеним списком, і `eq` впав би на не-атомі); `fire-rule-on-working-memory` — те саме напряму з `*working-memory*`. Крок 3 зроблено: `fire-rules-on-facts` — застосування **всього набору правил** до списку фактів за один прохід (перебір усіх правил через `append`, той самий fan-out паттерн, що й `prove-goal` у `lib/reason.my`); `fire-rules-on-working-memory` — те саме напряму з `*working-memory*`. Крок 4 зроблено: `run` — fixpoint-цикл, повторно застосовує весь набір правил, дедуплікуючи нові факти через `equal?` (`member?`/`append-new`), поки прохід не додає нічого нового. `run` — чиста функція, що повертає фінальний список фактів, а не мутує `*working-memory*` зсередини циклу — та сама причина, що й функціональний `count-usage`: `run` це `lambda`, а `def` мутує лише свій фрейм, тож тіло циклу не може стійко оновлювати глобальний стан по ітерації. Синхронізація назад — окремим макросом `assert-facts!`, викликаним на верхньому рівні: `(assert-facts! (run rules *working-memory*))`. Свідомо ще без retract/truth maintenance (факти лише додаються, ніколи не видаляються) — наступний крок, окремо узгоджується з користувачем перед стартом, не мандат рухатись автономно далі. Тести: `fire_rule_produces_a_new_fact_when_the_pattern_matches`, `fire_rule_returns_no_match_when_the_pattern_fails`, `fire_rule_on_facts_collects_new_facts_and_drops_non_matches`, `fire_rule_on_facts_returns_empty_list_when_nothing_matches`, `fire_rule_on_working_memory_reads_the_global_fact_list`, `fire_rules_on_facts_applies_every_rule_and_collects_all_results`, `fire_rules_on_working_memory_reads_the_global_fact_list`, `run_reaches_a_fixpoint_by_chaining_rules_across_passes`, `run_does_not_loop_forever_when_a_rule_reproduces_an_existing_fact`, `assert_facts_merges_run_results_into_the_global_working_memory`, `assert_fact_adds_to_the_global_working_memory` у `crates/my-lisp/tests/forward.rs`.

## Нотатки

- Гілка `my-lisp` у `my-idea`-репозиторії (origin) зараз ще існує (і локально, і на GitHub) — користувач раніше питав, чи прибрати її, раз є окремий репо. Рішення не ухвалене — уточнити на початку нової сесії.
- `private/` папка (PROJECT_MEMORY.md, PROFILE.md) з `my-idea` **не повинна** потрапити в новий репо — вона й так у `.gitignore`, але перевірити після filter-repo, що жоден слід не потрапив у git-історію.
