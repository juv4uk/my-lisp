# Cross-dialect comparison — сім примітивів my-lisp проти історії Lisp

**Статус:** PROPOSED (документ, не implementation). Побудовано методом
Волта Діснея (Мрійник → Реаліст → Критик, 2026-08-28, той самий
вечір, що й `PROPOSAL-INVIOLABLE-PRIMITIVES.md`). Критик перевірив
кожне історичне твердження напряму проти першоджерел, не повірив на
слово попереднім етапам.

**Зв'язок з CANON**: цей документ — окремий, доповнюючий шар, НЕ
частина самого CANON-механізму (`PROPOSAL-INVIOLABLE-PRIMITIVES.md`).
CANON лишається малим і одноголосим (лише значення my-lisp); тут —
порівняльний контекст, на який CANON лише посилається.

**Живе середовище — оновлено 2026-08-28, того ж вечора**:
`guile`/`racket`/`emacs`/`clojure` встановлено через Guix
(`guix install guile racket emacs clojure`) і реально протестовано
живцем нижче — ці чотири рядки тепер `empirically confirmed`
(рівень: *local run*, за драбиною доказовості §3a — не *clean CI run*,
не незалежне відтворення). `sbcl`/`picolisp`/InterLisp лишаються
`source-confirmed`/`predicted` — не встановлені.

---

## Чотири осі порівняння (прив'язані до семи примітивів my-lisp)

1. **NIL/false/empty-list**: об'єднано в одне значення, чи розділено.
2. **Гранулярність `eq`**: один предикат тотожності, чи кілька рівнів.
3. **Статус `cond`/`quote`**: спеціальна форма (evaluator/reader-рівень) чи звичайна процедура/макрос.
4. **`car`/`cdr` на порожньому вводі**: помилка чи дозвільне повернення nil.

## Живе ядро (8 діалектів)

| Діалект | Вісь 1 (NIL/false/'()) | Вісь 2 (eq) | Вісь 3 (cond/quote) | Статус | Джерело |
|---|---|---|---|---|---|
| **LISP 1.5 (1960)** | Об'єднано: nil = false = порожній список | один `eq` (атомна тотожність) | обидва — спеціальні форми | source-confirmed | McCarthy 1960/1962 manual (вторинні джерела, узгоджені; первинний скан не прочитано напряму) |
| **Common Lisp (ANSI)** | Об'єднано, як LISP 1.5 | `eq`/`eql`/`equal`/`equalp` — 4 рівні | обидва — спеціальні форми | source-confirmed | CLHS glossary: "nil is ... the empty list, ... representing false, and the name of the empty type" |
| **Scheme (R7RS)** | **Розділено на ТРИ речі**, не дві: `#f` ≠ `'()` ≠ символ `nil` (якщо він взагалі є) | один `eq?` (плюс `eqv?`/`equal?`) | обидва — спеціальні форми | empirically confirmed (local run, через Guile та Racket — обидві R7RS-сумісні) | `(eq? '() #f)` → `#f`; `(if '() "true" "false")` → `"true"` — перевірено живцем і в Guile, і в Racket, ідентичний результат |
| **Emacs Lisp** | Об'єднано, як LISP 1.5 (`nil` = symbol = false = `'()`, ідентичні) | один `eq` | обидва — спеціальні форми | empirically confirmed (local run) | `emacs --batch --eval`: `(eq (quote ()) nil)` → `t`; `(if (quote ()) "true" "false")` → `"false"`; `(car nil)` → `nil` |
| **Guile (Scheme-гілка)** | Як Scheme (розділено) | як Scheme | обидва — спеціальні форми | empirically confirmed (local run) | `(eq? '() #f)` → `#f`; `(if '() "true" "false")` → `"true"` |
| **Clojure** | `nil` ≠ `false`, обидва falsy (не unified, не повний split) | `identical?` (тотожність) окремо від `=` (значеннєва рівність) — **не** `eq` (Clojure має свій `eq`, тонко інший для деяких числових/nil випадків) | `cond` — макрос над `if`; `quote` — спецформа | empirically confirmed (local run, через `java -cp clojure.jar clojure.main`) | `(= nil false)` → `false`; `(identical? nil false)` → `false`; `(nil? false)` → `false`; `(false? nil)` → `false`; `(if nil "true" "false")` → `"false"` |
| **InterLisp** | Не про цю вісь напряму — про вісь 4 | — | — | **predicted, низька впевненість** | Вторинний переказ (Interlisp-VAX Users Manual), первинний PDF (bitsavers) не прочитано напряму; не підтверджено для Interlisp-10/Xerox D-machine специфічно |
| **PicoLisp** | `NIL`/`T` — окремі, розрізнені через тегування в нижніх 4 бітах вказівника комірки (не окреме поле) | — | — | source-confirmed | Офіційна документація, software-lab.de/doc/ref.html |

**Вісь 4 (`car`/`cdr` на порожньому вводі)**: InterLisp — єдиний
кандидат "живого ядра" на дозвільну поведінку (`(CAR NIL)` = `NIL`,
не помилка), протиставлений суворішій поведінці CL/Scheme. **Лишається
predicted/низька впевненість** — знайдено лише вторинне джерело, не
першоджерело. Альтернативи не знайдено (Critic шукав спеціально) —
не вигадувати заміну, лишити позначку чесною, доки хтось не відкриє
первинний Interlisp-VAX/Interlisp-10 manual напряму.

## Додаток (не "живе ядро", але реально існували)

MacLisp, InterLisp-наступники, Franz Lisp, NIL, Spice Lisp, S-1 Lisp,
ZetaLisp/Lisp Machine Lisp (+ LMI, TI Explorer), Le-Lisp, T, AutoLisp,
EuLisp, ISLISP, MIT Scheme/Chez Scheme/Racket/Arc (Scheme-гілка, уже
представлена Guile), newLISP, LFE, Shen, Hy, Fennel, Janet, Carp —
історично реальні, але або надлишкові щодо осі, яку вже покриває
запис ядра, або занадто слабо задокументовані для просування зараз.

## Дисципліна тверджень (не новий винахід — §3/§3a кореневого CLAUDE.md)

Кожен рядок вище позначений `predicted`/`source-confirmed`/
`empirically confirmed`. Станом на 2026-08-28 (той самий вечір) чотири
рядки (Scheme/R7RS, Emacs Lisp, Guile, Clojure) — `empirically
confirmed`, рівень *local run* за драбиною доказовості §3a (ручна
одноразова перевірка в цій сесії, ще не *clean CI run*, тим більше не
незалежне відтворення іншим середовищем/особою). LISP 1.5, Common
Lisp, InterLisp, PicoLisp лишаються `source-confirmed`/`predicted` —
жоден історичний чи ANSI CL рантайм, і жоден з InterLisp/PicoLisp, не
встановлений і не протестований у цьому середовищі. Коли (якщо)
з'явиться виконуваний `tests/fixtures/dialect-comparison.my` —
твердження переперевіряються ПРИ ЧИТАННІ (перезапуском фікстури проти
того, що реально встановлено), не за збереженою датою "перевірено
колись" — той самий механізм §3a, що вже керує кожним іншим
твердженням цієї екосистеми, не паралельна схема.

**Явно не зроблено**: сам виконуваний фікстур-файл
(`tests/fixtures/dialect-comparison.my`) ще не написаний — чотири ручні
перевірки вище не консолідовані в жоден скрипт чи checked-in тест;
підняти цей local-run рівень до clean-CI-run рівня доказовості означає
саме написати цей фікстур-файл і запустити його в CI. Це — наступний
крок, не зроблений цим редагуванням.
