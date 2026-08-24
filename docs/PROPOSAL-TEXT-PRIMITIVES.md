# PROPOSAL: мінімальні текстові примітиви — string-slice, argv

**Статус:** PROPOSED v2.2 · **Дата:** 2026-08-25 · **Автор:** Vyasa (COMPILER STEWARD)
**v2 корекція:** `read-file-string` ВЖЕ ІСНУЄ як host-capability `read-file`
(my-lisp-host/lib.rs:723-726 разом з read-dir/read-file-bytes/write-file),
і поточний CLI вже викликає `my_lisp_host::install()` перед запуском скрипта;
попередній claim про відсутність installation спростовано live-кодом.
**Тип:** ядро → 2 нові builtins (minor bump; surface розширення, семантика наявних форм не змінюється)
**Драйвер:** директива власника «максимально переводимо екосистему на my-lisp» —
міграційна хвиля вперлася у відсутність рядкового I/O (пілот
scripts/program-symbol-table.my, BLOCKED; evidence у комміті)

---

## 1. Дефікт

Мова не має способу прочитати файл як РЯДКИ або взяти підрядок за
індексом. Наявна строкова бібліотека — char-рекурсія над string-first/
string-rest (core.my:347+) без індексного доступу; `read-all` парсить
s-вирази і непридатний для не-Lisp форматів (.asm/.inc/.yaml/.log).
Наслідок: міграційна хвиля блокована без індексних зрізів/argv (нові
builtins); файловий `read-file` уже доступний через host capability.
Пілот-доказ: scripts/program-symbol-table.my BLOCKED commit.

## 2. Пропонована поверхня (мінімум, без дублювання)

| Пункт | Тип | Семантика |
|---|---|---|
| `string-slice` | новий builtin | s start end → string; CHAR-індекси (узгоджено з string-first/rest); start≥end → ""; поза межами → clamp |
| `*argv*` | новий builtin | () → list-of-strings аргументів після скрипта; без аргументів → () |

`read-line` не пропонується; `read-file-string` ВИЛУЧЕНО з v2 — read-file
уже існує як capability, дублювати поверхню немає сенсу.

### §1a Correction: CLI capability parity already holds
Перевірка live-коду [VERIFIED 2026-08-25]: `crates/my-lisp-cli/src/main.rs`
викликає `my_lisp_host::install()` до створення сесії та запуску скрипта.
Тому `read-file` та інші зареєстровані host capabilities доступні CLI так
само, як і host embedder. Попередній claim про відсутність CLI installation
був stale і вилучений з proposed surface; окремої capability-роботи тут не
потрібно.

## 3. Doctrine: чому це виправдовує нові примітиви

«Library before core primitive» каже: спершу бібліотека. Але бібліотека
не може взяти зріз або отримати CLI-аргументи без цих двох атомів — це той самий
клас мінімальних машинних машинних примітивів, що й car/cdr/read.
Прецедент: `json-parse` додано бо `\uXXXX` неможливий у .my (yantra.my).

## 4. Що НЕ входить
- regex, split-by-separator, line-reader з буферизацією — бібліотека поверх
- запис файлів (окрема пропозиція, capability-gated)
- argv як мутований стан — чисте значення на старті


## 4.5 ABI / портативність
- Індекси string-slice — за СИМВОЛАМИ (Unicode scalar values), не байтами:
  узгоджено зі string-first/string-rest; реалізація через char_indices,
  O(n) на виклик — задокументовано.
- Кодування: рядки мови = UTF-8 Rust String; read-file декодує strict UTF-8,
  invalid → named error (не lossy).
- Портативність: примітиви не нормалізують шляхи й роздільники — це
  відповідальність викликуча; жодних OS-specific гілок у ядрі.
- Capability-взаємодія: string-slice/*argv* — чисті, без I/O, тому БЕЗ
  capability-gate; файлові операції лишаються за capability-моделлю.

## 4.6 Conformance fixtures (до contracts/fixtures після ратифікації)
1. `(string-slice "привіт" 1 3)` → "ри"            ; char-index на кирилиці
2. `(string-slice "abc" 0 0)` → ""                  ; порожній
3. `(string-slice "abc" 2 9)` → "c"                 ; clamp end
4. `(string-slice "abc" 4 9)` → ""                  ; повний clamp
5. `(*argv*)` з аргументами `[x y]` → "("x" "y")"   ; список рядків
6. `(*argv*)` без аргументів → "()"
7. read-file відсутнього шляху → named error (існуюча поведінка, фіксувальна)

## 5. Тест-план
1. unit: slice межі (0/клімп/порожній), utf-8 багатобайтові границі по КОДАх символів не байтам
2. integration: program-symbol-table pilot завершується і дає parity vs symbol_table.py
3. conformance: fixtures у tests/fixtures/conformance.my

## 5.1 Empirical motivation

The migration decision is benchmark-driven, not a convenience request. On
2026-08-25 the current self-hosted `check-stale-refs.my` and Python reference
checked the same live repository state with matching success status. Twenty
fresh processes measured Python at **0.89 s / 12,020 KB** and my-lisp at
**14.00 s / 3,328 KB**. A decomposition measured 20 empty my-lisp processes at
**0.06 s**, versus **13.65 s** for the checker. Therefore process startup is
not the bottleneck: character-recursive text traversal/string construction is.
`string-slice` is proposed as the smallest contract-level experiment that can
replace that path; Python remains the reference until the fixtures and parity
gate pass.

## 6. Альтернативи (відхилені)
- Залишити клас утиліт у Python-bootstrap → суперечить напряму власника
- Повний POSIX-I/O пакет → надлишково, capability-хвиля пізніше
- (read-line) замість read-file-string → зайвий стан у readerʼі

## 7. Очікуваний ефект
Розблокування A1 пілота + всієї лінійки «утиліта міграції» (stale-refs,
dependency-валідатори); прибирає системного блокера директиви міграції.
