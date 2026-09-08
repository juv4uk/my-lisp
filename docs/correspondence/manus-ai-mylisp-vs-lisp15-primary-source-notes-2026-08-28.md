# Нотатки першоджерела: paper Маккарті 1960 р. про Lisp

**Статус:** супровідний доказовий документ до
`manus-ai-mylisp-vs-lisp15-comparison-2026-08-28.md` (Manus AI,
2026-08-28) — точні джерельні посилання й цитати, на яких базується
основне порівняння my-lisp з LISP 1.5.

## Source identity

Канонічний архівований HTML — власна Stanford-версія Джона Маккарті
**"Recursive Functions of Symbolic Expressions and Their Computation
by Machine, Part I"**, опублікована в *Communications of the ACM* у
квітні 1960. [1] Тісно пов'язаний AI Memo `AIM-008` (MIT, березень
1959) — явно машинно-незалежний попередник; опис архіву каже, що
storage representation і machine subroutines мали додатись пізніше. [2]

## Exact technical propositions used in the comparison

| Секція | Основне твердження | Релевантність для порівняння |
|---|---|---|
| Introduction | Lisp виник для маніпуляції декларативними й імперативними реченнями заради дедукцій Advice Taker, потім спрощений до машинно-незалежних рекурсивних функцій символьних виразів. [3] | Розділяє AI-намір Маккарті від компактного мовного механізму. |
| §2 | Часткові функції й незавершення — звичайні обчислювальні факти; умовні вирази роблять рекурсивне визначення зручним. [4] | Порівнює поточну стратегію помилок/ресурсів/незавершення, не лише синтаксис. |
| §3a-d | S-вирази — атоми плюс ordered pairs; списки й dotted lists — нотація. Елементарна основа: `atom`, `eq`, `car`, `cdr`, `cons`; рекурсивні функції виникають через композицію, умовність і рекурсію. [5] | Пряма відповідність семантичного ядра. |
| §3e-g | Функції можуть самі бути представлені S-виразами; `apply`/`eval` формують універсальну S-функцію з `quote`, `cond`, `lambda`, `label`, association-list environment і функціями вищого порядку. [5] | Пряме порівняння з поточними values, closures, межею macro/data, evaluator і environment. |
| §4a-c | Машинні list structures дозволяють спільне використання; цикли навмисно заборонені; автоматичне повернення пам'яті позначає досяжні cells від фіксованих base registers, коли free storage вичерпано, потім повертає недосяжні у free list. [6] | Розділяє семантичні values від 704 cell representation, визначає історичний корінь M0 GC. |
| §4e-f | Система 1960 вже поєднує interpreter, reader/printer, diagnostics/tracing і опційні compiled functions; compiled S-functions — приблизно 60× швидші за interpreted. [6] | Порівнює поточний interpreter, FASL/cache і намічений CML/FPGA шлях, не переносячи число швидкодії 1960 року як таке. |
| §5 | Маккарті явно порівнює іншу лінійну репрезентацію з list structure: вибір представлення виразу міняє вартість примітивної операції, навіть коли обидва формалізми обчислювально універсальні. [7] | Релевантно для UPC/FASL/value-representation дизайну: універсальна виразність не стирає економіку представлення. |
| §6 | Single-entry/single-exit flowchart трактується як трансформація state-vector і в принципі може бути виражений через рекурсивні функції. [8] | З'єднує поточні lowering/FPGA/CML амбіції зі старою, але досі центральною ідеєю: control flow як функція над явним машинним станом. |

## Critical historical guardrail

Papір — не "джерело кожної пізнішої фічі Lisp" і не опис сучасної
lexical, гігієнічної macro-системи. Це точна специфікація 1960 року
одного формалізму рекурсивних S-функцій та реалізації на IBM 704.
Порівняння мусить розрізняти **спадкоємність ідеї** від
**ідентичності фічі**.

## References

[1] https://www-formal.stanford.edu/jmc/recursive.html — McCarthy archive index: original Lisp paper
[2] https://dspace.mit.edu/entities/publication/b66ae9e2-cc26-4735-a4b7-02cfbe6b0ce6 — MIT DSpace: AIM-008, 13 March 1959
[3] https://www-formal.stanford.edu/jmc/recursive/node1.html — Paper introduction
[4] https://www-formal.stanford.edu/jmc/recursive/node2.html — Functions and function definitions
[5] https://www-formal.stanford.edu/jmc/recursive/node3.html — Recursive functions of symbolic expressions
[6] https://www-formal.stanford.edu/jmc/recursive/node4.html — The LISP programming system
[7] https://www-formal.stanford.edu/jmc/recursive/node5.html — Another formalism for functions of symbolic expressions
[8] https://www-formal.stanford.edu/jmc/recursive/node6.html — Flowcharts and recursion
