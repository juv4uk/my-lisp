# LSP M3 (design) — Eval-time diagnostics через oracle

**Статус:** DESIGN — не реалізовано. Цей документ фіксує міркування
2026-08-22, щоб наступна сесія не вигадувала їх заново.
**Автор:** Сакші (sākṣī, ox-alpha)
**Звʼязок:** [[PROPOSAL-FIRST-CLASS-BUILTINS]] · `docs/lsp-m0.md` · oracle `:9999`

---

## 1. Проблема: межа parse-only діагностик

M0–M2 діагностики працюють тільки на канонічному парсері:

```lisp
(def x 1
;; ✗ "unclosed list", span від парсера — дешево і чесно
```

Парсер відповідає лише на «чи це синтаксично коректний my-lisp?».
Наступні помилки він принципово не бачить:

```lisp
(+ 1 "hello")       ;; типова помилка — проявиться тільки при eval
(undefined-fn 5)    ;; невідомий символ у head/argument позиції
(/ 1 0)             ;; ділення на нуль (DivisionByZero від contract 3.0)
(car 42)            ;; застосування не-списку
```

## 2. Джерело правди: oracle `:9999`

my-lisp вже має TCP-оракул з op `eval`, який реально виконує код:

```text
(request (id N) (op eval) (source "..."))
→ (response (status ok|error) (kind ...) (message ...) ...)
```

Ідея фази 6: LSP надсилає документ (або його форму) в оракул,
отримує runtime-помилку і публікує її як diagnostic.

## 3. Чому це найскладніша фаза (шість пасток)

| # | Пастка | Наслідок без обробки |
|---|---|---|
| 1 | Код **виконується** на кожен keystroke | print-сміття, side effects, небезпека |
| 2 | Нескінченні цикли / рекурсія без бази | oracle висне назавжди |
| 3 | Документ напівготовий | що саме евалювати? |
| 4 | Дві інстанції мови (editor vs oracle env) | стан розходиться |
| 5 | Oracle не повертає span | звідки взяти range для diagnostic? |
| 6 | Eval повільний (парс = µs, eval = s) | UI замерзає без debounce |

## 4. Проєктні рішення

### 4.1. Тригер — didSave, ніколи keystroke

Діагностика рахується тільки на явному збереженні. Keystroke-діагностики
лишаються parse-only (фаза M0). Це знімає пастки 1 і 6 майже повністю.

### 4.2. Свіже середовище на кожен прогін

Oracle-сесія для діагностики стартує з чистою глобальною інстанцією.
Ніякого накопиченого стану між прогонами → пастка 4 зникає; ціна —
«невідомий символ» для речей, визначених у ІНШИХ файлах workspace.
Рішення: конкатенувати defs усіх файлів workspace index у порядку
топологічному за залежностями (поки що: алфавітний порядок файлів,
кожну форму окремо), і евалювати їх перед цільовим документом.

### 4.3. Поформовий eval → точний span БЕЗ евристики

Ключове спостереження: канонічний парсер дає span кожної top-level форми.
Замість «надіслати весь файл» надсилати **по одній формі**:

```text
для кожної top-level форми F (span від парсера):
    response ← oracle.eval(F.text)
    якщо error → diagnostic зі span = F.span   ← точний, без мапінгу!
    (наступні форми все одно пробуємо: помилки незалежні)
```

Пастка 5 зникає: span діагностики = span форми, яку ми самі відправили.
Обмеження чесно задокументувати: помилка всередині великої форми
вкаже на всю форму, а не на точний рядок.

### 4.4. Timeout + ізоляція ефектів

- Обгортка навколо oracle-виклику: timeout 1000 ms на форму;
  таймаут → diagnostic «evaluation timeout (possible infinite loop)».
- Diagnostic-режим оракула мусить виконуватись у **pure core**
  (physical core/host split, коміт f565f66): host-capability (файли,
  процеси, tcp) вимкнений. `(read-file ...)` у діагностиці = помилка
  «host capability disabled in diagnostics», а не реальне читання.
- Паралельно з GPU-задачами не поєднувати: oracle CPU-bound, але
  легкий; конфліктів нема.

### 4.5. Формат diagnostic

```json
{
  "range": <span top-level форми>,
  "severity": 1,
  "source": "my-lisp-eval",
  "message": "<kind>: <message> [eval]"
}
```

`source` відрізняє runtime-діагностики від parse-only (джерело
`"my-lisp"`). Обидва списки публікуються разом.

## 5. Архітектурна межа (не порушувати)

| Concern | Власник |
|---|---|
| Eval семантика | canonical core через oracle — LSP не має власного evaluator'а |
| Span форм | canonical parser (`parse()` spans) |
| Timeout / sandbox policy | LSP adapter (`diagnostics_eval.rs`) |
| Мапінг форма→span | LSP adapter (тривіальний: span уже є) |

Новий модуль: `crates/my-lisp-lsp/src/diagnostics_eval.rs` +
оракул-клієнт поверх `transport.rs`-стилю framing (одноразовий
one-shot TCP, як `oracle.rs` у my-idea).

## 6. Порядок реалізації

1. Оракул-клієнт у LSP crate (one-shot, framed, timeout).
2. `diagnostics_eval.rs`: поформовий прогін одного документа,
   свіже env, timeout, pure-core режим.
3. Інтеграція в `server.rs`: didSave → merge(parse-diags, eval-diags).
4. Конфіг: `evalDiagnostics: on-save | off` (default off до стабільності).
5. Workspace-рівень: попереднє евалювання defs інших файлів (4.2).
6. Тести: e2e з живим оракулом (mark `#[ignore]` якщо оракул недоступний),
   включно з timeout-кейсом і pure-core відмовою.

## 7. Відкриті питання

1. Чи потрібен окремий порт/режим оракула для діагностики (ізоляція від
   продакшн-запитів WSM-24 тощо)?
2. Чи показувати результати УСПІШНИХ форм (severity Hint)? Зараз — ні.
3. Batch-size: скільки форм максимума на один save (захист від гігантських
   файлів)? Пропозиція: перші 200 форм, решта — parse-only.

---

*Див. також: physical core/host split — `git log -1 f565f66`
(«capability-free core is now literally true») — саме він робить
pure-core діагностику можливою.*
