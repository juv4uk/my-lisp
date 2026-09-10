# Рекомендації задач: Cyberpunk + CML export (2026-09-10)

Статус: **advisory**. Не змінює `tasks.my` (щоб не ламати oracle-check). Власник може перенести рядки в `tasks.my` вручну.

Повна крос-репо карта: [cml/…/CROSS-REPO-TASK-RECOMMENDATIONS-2026-09-10.my](https://github.com/juv4uk/cml/blob/master/evidence/cyberpunk/CROSS-REPO-TASK-RECOMMENDATIONS-2026-09-10.my).

## my-lisp (це репо)

| ID | Priority | Задача |
|----|----------|--------|
| **CP-EXPORT-ARTIFACT-PIN** | 9.5 | Закомітити детермінований `mylisp-cml-export.wsm` (або узгоджений шлях) з `cml-export`, щоб cml міг hard-pin FNV digest замість `pending-producer-byte-pin`. |
| **CP-FIXTURES-OWNED** | 9.0 | `docs/cyberpunk-host-dispatch-fixtures.md` лишається oracle; зміни хоста спочатку в fixtures. |
| **CP-MULTI-HOST-CAPABILITY-MODEL** | 8.5 | Дизайн: CLI (blocking) vs WASM vs game (frame/callback) для capabilities — **до** другого host-specific набору. |
| **CP-ORACLE-IN-PROCESS-GAP** | 8.0 | Явно назвати розрив CI oracle ≠ in-game evaluator; запропонувати hash артефакту dll або спільний fixture runner. |
| **CP-SLICE2-ON-REQUEST** | 6.0 | Не розширювати semantic export slice-2, доки cml не попросить конкретний program shape. |

## Вже зроблено (не дублювати)

- Semantic export producer `cml-export` (slice-1: quote/cond/lambda/define/eq/−).
- Cyberpunk fixtures + paradigm-fit doc.
- Питання delivery/next-slice для export: cml відповів (committed artifact; cml picks slice 2).

## Не робити тут

- RED4ext / Win64 DLL — **wsm-my-lisp**.
- Продуктовий README scope — **my-lisp-cyberpunk** + власник.
- Runtime `eval_string` у **cml** — non-goal.
