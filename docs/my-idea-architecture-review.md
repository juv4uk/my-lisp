# `my-idea` architecture review (2026-08-11)

Not a `my-lisp` design document — `my-idea` is a separate repository (the IDE this language was originally extracted from, per the root `README.md`). Saved here because part of the review — the Cargo-dependency vs. git-submodule revision-sync risk between `my-idea` and `my-lisp` — is directly relevant to this ecosystem's conformance work (`docs/ecosystem-roadmap.md`, `docs/ecosystem-sync.md`) even though the rest of the review (ClojureScript module structure, state management, renderer) is `my-idea`-internal and out of scope for this repository to act on.

Author: the user, in conversation with this session, 2026-08-11. Recorded verbatim below.

---

`my-idea` зараз виглядає як окремий продукт із власною логікою, а не просто GUI для `my-lisp`. І це, на мою думку, правильний напрям.

Найсильніше рішення вже зафіксоване в README: IDE призначена для звичайного програмування, а `my-lisp` живе всередині як Language Lab, тобто IDE не обмежується однією мовою. Основа — CodeMirror 6, ClojureScript UI, Tauri v2 + Rust, а `my-lisp` підключена як незалежна мова.

Архітектурно зараз це приблизно так:

```text
                    my-idea
                       │
        ┌──────────────┼──────────────┐
        ▼              ▼              ▼
      Editor        Workspace      Language Lab
        │              │              │
    CodeMirror     files/projects     │
        │              │              ▼
        │              │          my-lisp
        │              │        ┌─────┴─────┐
        │              │        ▼           ▼
        │              │      WASM        Rust
        │              │       web         Tauri
        └──────────────┴──────────────┬──────┘
                                      ▼
                                      UI
```

І фронтенд уже непогано розкладений: `core.cljs`, `editor.cljs`, `workspace.cljs`, `preview.cljs`, `wasm.cljs`.

## Що мені особливо подобається

`editor.cljs` практично не знає про решту програми. Він займається CodeMirror: мовними режимами, completion, lint gutter, темою, source/update. Навіть evaluator передається опосередковано через `diagnose-fn` і callbacks. Це хороший модульний дизайн.

`wasm.cljs` теж дуже вузький: завантажити WASM, сказати `ready?`, `failed?`, `evaluate`, `diagnose`. Він не намагається керувати IDE.

`workspace.cljs` містить модель документів і значну частину файлової абстракції:

```clojure
:update-active
:open-document
:close-document
:language-mode
```

Частина цих функцій є звичайними перетвореннями immutable Clojure-мапи, тобто тут уже природно використовується функціональний стиль.

На Rust/Tauri боці теж хороший принцип: runtime не отримує довільний доступ до системи. Файлові операції обмежені workspace, а `safe_existing` перевіряє, що шлях після canonicalization не втік за корінь workspace. `evaluate_my_lisp` запускає capability-free `Session::default()`.

Це дуже добре узгоджується з тією ж філософією, яка зараз формується у `my-lisp`: можливості мають бути явними, а не магічними.

## Найбільша проблема зараз — `core.cljs`

Тут уже видно майбутній технічний борг.

`core.cljs` має приблизно 18 KB і одночасно відповідає за:

```text
global UI state
localization
themes
workspace commands
file opening/saving
evaluation
language selection
HTML rendering
event wiring
```

Тобто він поступово стає god module.

Зараз це ще не катастрофа. Проєкт маленький, і такий файл дозволяє швидко рухатися. Але якщо додати:

```text
formatter
linter
diagnostics
System Observatory
git integration
search
command palette
settings
plugin/language adapters
```

`core.cljs` дуже швидко стане місцем, де «все знає про все».

Я б почав розділяти його до цього моменту.

Наприклад:

```text
my_idea/
├── core.cljs          ; bootstrap only
├── state.cljs
├── commands.cljs
├── ui.cljs
├── i18n.cljs
├── theme.cljs
├── editor.cljs
├── workspace.cljs
├── runtime/
│   ├── protocol.cljs
│   ├── wasm.cljs
│   └── tauri.cljs
└── observatory/
```

Тоді `core.cljs` залишиться приблизно:

```clojure
(init-state!)
(init-editor!)
(init-runtime!)
(render!)
```

а не всією програмою.

## Друге: state зараз глобальний, але це не обов'язково погано

Ти маєш:

```clojure
(defonce state
  (atom {...}))
```

Для ClojureScript IDE такого масштабу це абсолютно нормальне рішення.

Я б не кидався ставити Redux/Re-frame чи щось важке.

Навпаки, хороший напрям:

```text
state atom
   +
pure update functions
   +
effectful commands outside
```

Тобто:

```clojure
(swap! state workspace/open-document ...)
```

— дуже хороший патерн.

Я б лише поступово зробив так, щоб `swap!` було менше розкидано по UI-коду.

Наприклад:

```clojure
(dispatch! [:document/open path])
(dispatch! [:theme/set "dark"])
```

але без складної event-framework.

## Третє: renderer зараз занадто ручний

`render!` генерує великий HTML через `str` і потім робить:

```clojure
(set! (.-innerHTML app) ...)
```

Це зараз працює і є дуже легким рішенням.

Але для майбутньої IDE це одна з найслабших частин.

Тому що коли UI стане складнішим:

```text
tabs
split panes
diagnostics panel
symbol explorer
System Map
World history
proof graph
compatibility matrix
```

ручне конструювання HTML-рядків стане болючим.

Я б при цьому не переходив одразу на React/Reagent лише тому, що «так прийнято».

Можна зробити власний маленький component layer:

```clojure
(defn tab-view [model] ...)
(defn sidebar-view [model] ...)
(defn console-view [model] ...)
(defn observatory-view [model] ...)
```

і лише потім, якщо реально знадобиться virtual DOM, вирішувати питання framework.

## System Observatory

Оце найцікавіша частина всього `my-idea`.

Останній великий коміт уже описує нове бачення: `my-idea` має стати System Observatory для екосистеми:

```text
my-lisp
compiler
fpga-lisp
    │
    ▼
my-idea
```

але принципово не четвертим джерелом істини. IDE має читати machine-readable contracts і показувати evidence того, де реалізації збігаються або розходяться. Коміт описує System Map, compatibility matrix, fixture як базовий UI-об'єкт, expression trace graph, Evidence Graph і timeline.

На мою думку, це дуже сильна ідея.

Бо IDE тоді перестає бути:

текстовий редактор + кнопка Run.

І стає:

інструментом спостереження за мовою та машиною.

Наприклад:

```text
                 cons
                  │
        ┌─────────┼─────────┐
        ▼         ▼         ▼
      Rust       CML       FPGA
       ✓          ✓          ✓
```

натискаємо `cons`:

```text
Fixture: G5/cons-03

Rust
 commit abc...
 result: PASS

Compiler
 commit def...
 result: PASS

FPGA
 commit 123...
 result: FAIL

difference:
expected: (a b)
received: ...
```

Оце вже унікальна особливість IDE.

VS Code такого не дає з коробки.

## І тут виникає зв'язок з Clean Code

Після нашої попередньої розмови я би `my-idea` зробив місцем, де філософія `my-lisp` стає видимою.

Наприклад редактор може показувати:

```text
world-tell     pure
reason-in-world pure

write-file!    effect
tcp-write!     effect
```

Функція з прихованим global state:

```text
⚠ hidden dependency: *knowledge-journal*
```

А довга вкладена функція:

```text
complexity
nesting 8
```

Але не червоним «ПОМИЛКА!».

М'яко:

```text
hint
This function may be easier to understand
if the inner condition is named.
```

Тоді відбувається цікава річ:

```text
my-lisp
  визначає
  хороший архітектурний шлях

my-idea
  робить його
  видимим програмісту
```

Це дуже сильна синергія.

## Один серйозний архітектурний ризик

Зараз `my-idea` залежить від `my-lisp` двома шляхами.

Rust/Tauri бере її як Cargo git dependency, а WASM build потребує фізичного checkout, тому `external/my-lisp` доданий як git submodule. Коміт прямо описує, що ці revision-и спеціально синхронізовані.

Це працює, але створює ризик:

```text
Cargo.lock
     ↓
my-lisp revision A

external/my-lisp
     ↓
revision B
```

і Web IDE та desktop IDE раптом виконують різні версії мови.

Я б зробив автоматичний CI invariant:

```text
Cargo my-lisp revision
        ==
submodule my-lisp revision
```

і build повинен падати, якщо це не так.

Це маленька перевірка, але дуже важлива для системи, де Rust/WASM conformance є фундаментальним принципом.

## Що я би не робив

Я б зараз не перетворював `my-idea` на універсальний аналог VS Code.

README правильно говорить:

normal programming first, Language Lab — special built-in feature.

Але «звичайна IDE» не означає:

```text
1000 extensions
remote containers
Docker
cloud sync
AI marketplace
20 SCM providers
```

Тоді проєкт втратить свою особливість.

Я бачу кращу нішу:

```text
         lightweight programming IDE
                    +
             language laboratory
                    +
            system observatory
```

Тобто достатньо хороший редактор для щоденної роботи, але надзвичайно хороший інструмент для розуміння програмної мови.

## Мій рекомендований порядок розвитку

Я б зараз рухав `my-idea` так:

1. Розрізати `core.cljs`, поки він ще керований: state/i18n/commands/view.
2. Залишити `workspace.cljs` як модель workspace, але відділити pure model від browser/Tauri effects.
3. Ввести маленький runtime protocol:

```text
evaluate
diagnose
format
lint
contract
```

і вже під нього підключати WASM/Tauri/майбутню FPGA.

4. Додати formatter/linter `my-lisp` як capabilities IDE, а не реалізовувати їх вдруге в ClojureScript.
5. Зробити System Observatory MVP дуже малим:

```text
Fixture | Rust | Compiler | FPGA
```

без красивих графів на першому етапі.

6. Кожна галочка повинна мати `evidence`, як уже сформульовано у vision-документі.
7. Потім уже AST/tree explorer, proof viewer, World history і branch visualization.

## Найцікавіше враження після перегляду обох репозиторіїв

Раніше це виглядало як:

```text
my-idea
   ↓
пишемо my-lisp
```

Тепер правильніше бачити:

```text
             my-lisp
                │
                │ semantic contract
                ▼
   compiler ── my-idea ── FPGA
                │
                │ observes
                ▼
           programmer
```

Тобто `my-idea` може стати не матір'ю мови, а її лабораторним приладом.

І це дуже хороше розділення ролей: `my-lisp` визначає істину мови; FPGA доводить, що цю істину можна реалізувати іншим способом; а `my-idea` дозволяє людині бачити, порівнювати й розуміти всю систему.
