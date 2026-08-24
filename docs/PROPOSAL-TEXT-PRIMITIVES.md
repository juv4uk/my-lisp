# PROPOSAL: мінімальні текстові примітиви — read-line, string-slice, argv

**Статус:** PROPOSED v2 · **Дата:** 2026-08-24 · **Автор:** Vyasa (COMPILER STEWARD)
**v2 корекція:** `read-file-string` ВЖЕ ІСНУЄ як host-capability `read-file`
(my-lisp-host/lib.rs:723-726 разом з read-dir/read-file-bytes/write-file),
але стандартний CLI його не встановлює → фактичний дефікт інший, див. §1a
**Тип:** ядро → 3 нові builtins (minor bump; surface розширення, семантика наявних форм не змінюється)
**Драйвер:** директива власника «максимально переводимо екосистему на my-lisp» —
міграційна хвиля вперлася у відсутність рядкового I/O (пілот
scripts/program-symbol-table.my, BLOCKED; evidence у комміті)

---

## 1. Дефікт

Мова не має способу прочитати файл як РЯДКИ або взяти підрядок за
індексом. Наявна строкова бібліотека — char-рекурсія над string-first/
string-rest (core.my:347+) без індексного доступу; `read-all` парсить
s-вирази і непридатний для не-Lisp форматів (.asm/.inc/.yaml/.log).
Наслідок: міграційна хвиля блокована подвійно — CLI без файлових
capabilities (§1a) та без індексних зрізів/argv (нові builtins).
Пілот-доказ: scripts/program-symbol-table.my BLOCKED commit.

## 2. Пропонована поверхня (мінімум, без дублювання)

| Пункт | Тип | Семантика |
|---|---|---|
| §1a CLI capabilities parity | інтеграція | стандартний `my-lisp script.my` встановлює ті самі host-capabilities, що й embedders: read-file/read-dir/read-file-bytes/write-file (джерело вже є: my-lisp-host/lib.rs:723-726) |
| `string-slice` | новий builtin | s start end → string; CHAR-індекси (узгоджено з string-first/rest); start≥end → ""; поза межами → clamp |
| `*argv*` | новий builtin | () → list-of-strings аргументів після скрипта; без аргументів → () |

`read-line` не пропонується; `read-file-string` ВИЛУЧЕНО з v2 — read-file
уже існує як capability, дублювати поверхню немає сенсу.

### §1a Деталі (найважливіший пункт)
Виявлено [VERIFIED 2026-08-24]: capability-реєстр має файлові примітиви,
але CLI-бінарник їх не інсталює → `(assoc "read-file" (env))` = ()
у script.my, тоді як fpga-lisp/check-stale-refs.my (через host) працює.
Різні точки входу = різна мова. Фікс: CLI викликає той самий
install-набір що host; це виправляння розколу, не нова влада.

## 3. Doctrine: чому це виправдовує нові примітиви

«Library before core primitive» каже: спершу бібліотека. Але бібліотека
не може прочитати файл чи взяти зріз без цих трьох атомів — це той самий
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

## 6. Альтернативи (відхилені)
- Залишити клас утиліт у Python-bootstrap → суперечить напряму власника
- Повний POSIX-I/O пакет → надлишково, capability-хвиля пізніше
- (read-line) замість read-file-string → зайвий стан у readerʼі

## 7. Очікуваний ефект
Розблокування A1 пілота + всієї лінійки «утиліта міграції» (stale-refs,
dependency-валідатори); прибирає системного блокера директиви міграції.
