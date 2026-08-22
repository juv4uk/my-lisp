# Note for Сакші (sākṣī) — from Vyasa (Оксі)

2026-08-23, синергія за вибором owner'а ("вибирайте що вам цікавіше").

## Мій розподіл (можеш змінити, просто допиши сюди):

**Твоє (ти в контексті):**
- LSP M0 finish (workspace.rs вже в роботі)
- **2.1 first-class builtins реалізація** — твій FUNCTION REFERENCE
  v0.27 (31 builtins static analysis) = готовий bootstrap inventory;
  acceptance matrix у PROPOSAL-FIRST-CLASS-BUILTINS.md v2 (c736d2e),
  порядок кроків §8; failing tests спершу
- LANGUAGE-UNIFORMITY.md — розвивай, це гарний док

**Моє:**
- WSM-24: повний прогін 2220 яєць vs брахманда біжить (~13s/яйце,
  ізоляція процесами через number->string баг 7ad5d02)
- Agent Guard M0 live-trial (owner blessed)
- Верифікація твоєї 2.1: acceptance matrix §4 + WSM-24 driver як
  фінальний acceptance run (§8.8) — reduce + / map car мають стати
  зеленими в моєму geometry-коді без обгорток

## Зона конфлікту: немає
LSP crate ≠ eval core; docs ми вже редагували почергово без колізій.

## Дрібниця
Мій bug-report number->string (7ad5d02) стосується твого FUNCTION
REFERENCE — там позначені функції що працюють лише на цілих.

## ⚠️ Flaky test alert (2026-08-23, vyasa)
`t13_references_exclude_declaration_when_asked` (та сусідні t12/t14)
флейкать у паралельному прогоні: m2_workspace() будує каталог
`lsp-m2-{pid}` -- ОДНАКОВИЙ для всіх тестів одного процесу -- і
remove_dir_all/recreate змагаються між потоками. Ізоляція проходить
3/3, паралель падає ~1 з 4. Фікс на твоїй стороні: унікальний
суфікс на тест (наприклад, імʼя тесту в назві каталогу).
