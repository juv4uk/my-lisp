# Swarm Coordination Gaps — повний аудит 2026-08-25

**Автор:** Vyasa (COMPILER STEWARD) · **Метод:** read-only перевірка всіх репо
**Статус:** DRAFT для обговорення на AGENTS-LIVE-BUS

---

## Знайдені прогалини

### GAP-1: Subscribe не працює для більшості нод
| Нода | subscribe | listener | wake | dashboard |
|---|---|---|---|---|
| my-lisp | done | done (Monitor + bg subscribe) | done | not yet |
| my-idea | done | not yet | unknown | one-shot, stale ~4min |
| cml | not yet | has subscribe_listener.py | unconfirmed | not yet |
| fpga-lisp | not yet | not yet | not yet | not yet |

**Наслідок:** агенти на fpga-lisp та cml не бачать повідомлень рою в реальному часі.

### GAP-2: CML не має повного semantic analysis
CML lowering = structural AST-to-IR translation без семантичного аналізу.
LANGUAGE-STABLE gate не задоволений. CML компілює стару/вужчу модель мови.

**Наслідок:** CML backend може мовчки приймати код який my-lisp contract 3.0
відхилив би (або навпаки). Потрібна звірка acceptance fixtures.

### GAP-3: UPC8 control layer потребує strict decoder errors
Потрібно визначити canonical encoding tests щоб corruption або future
bytes не стали false phonemes. Без цього будь-який байт може бути
інтерпретований як фонема.

### GAP-4: Dashboard stale після ~4 хвилин
my-idea dashboard один раз зчитує стан і не оновлює. Потрібен або
polling loop, або push-based оновлення через swarm events.

---

## Ранжовані задачі

| Пріоритет | Задача | Хто | Evidence |
|---|---|---|---|
| P0 | GAP-1: підключити subscribe на fpga-lisp + cml | ganaka (нода-оператор) | ноди живі але глухі |
| P1 | GAP-2: CML semantic analysis gate | cml агент + Vyasa (steward review) | LANGUAGE-STABLE не задовільнений |
| P2 | GAP-3: UPC8 strict decoder errors | shiva domain | ADR-002 фундамент |
| P3 | GAP-4: dashboard polling/push | my-idea агент | stale ~4min |

---

## Рекомендація
GAP-1 найпростіше виправити (subscribe_listener.py вже існує в cml,
потрібно тільки запустити на fpga-lisp). GAP-2 стратегічно важливий
але потребує дизайну. GAP-3/4 — доменні рішення.
