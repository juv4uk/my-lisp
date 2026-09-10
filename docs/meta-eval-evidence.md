# Межа self-hosting для `lib/meta-eval.my`

Цей документ є людською проєкцією машинно-читаної матриці `knowledge/meta-eval-evidence.wsm`. Він **не** створює нової семантичної влади: authoritative status кожного row живе в `.wsm` matrix і перевіряється `scripts/check-meta-eval-evidence.py`.

> **Тимчасове правило до TASK-008:** цей Markdown не дублює вручну поточну таблицю статусів. Попередня ручна таблиця вже відстала від machine evidence (`macro-arity`, bare lookup, SCC, later binding), тому її видалено замість чергової синхронізації вручну. Наступний крок — генерувати human projection із `.wsm`.

## Дозволена лексика

- `metacircular evaluator exists` — дозволено: `my-eval` виконує репрезентативні my-lisp програми як Lisp-функція.
- `self-hosting witness` — дозволено лише для явно підтвердженого subset.
- `partial self-hosting` — дозволено, поки gaps названі й мають executable witnesses.
- `complete self-hosting` — **заборонено**, доки хоча б один required row має `partial`, `broken` або `unknown`.

## Named errors: correspondence, а не текстова тотожність

У репозиторії є paired proofs, де reference-side `ErrorKind` перевіряється окремо, а Lisp meta-evaluator повертає data-level observation. Підтверджені mappings у covered corpus:

```text
UnknownSymbol ↔ unbound-symbol
Type          ↔ not-callable
Arity         ↔ arity
InvalidForm   ↔ invalid-form
```

Ця нормалізація не дозволяє стирати відмінності `ErrorKind`: Rust kind спочатку перевіряється як контрактна категорія, і лише потім порівнюється з відповідним Lisp-data результатом.

Для **актуального переліку confirmed/partial/broken/unknown rows** треба читати `knowledge/meta-eval-evidence.wsm`, а не цей Markdown. Це навмисно усуває друге вручну редаговане джерело статусів до появи generated projection.

## Правило інтерпретації

Differential mismatch — це **finding**, а не автоматично «meta-eval неправильний». Якщо reference runtime суперечить `language-contract.my` або ратифікованому ADR, під підозрою reference implementation. Матриця лише обмежує силу тверджень тим, що реально пережило executable comparison.

## Поточний напрям

Після ADR-010 evaluator має внутрішній outcome/provenance channel: operator оцінюється перед аргументами, ordinary arguments — зліва направо, а перша evaluator failure зупиняє подальше evaluation. Error-shaped і fail-shaped quoted user data не є control-flow failure. Executable evidence для цього живе в `crates/my-lisp/tests/meta_eval_evidence.rs` та `crates/my-lisp/tests/meta_eval_error_provenance.rs`.

Документ знову може містити повну status table лише коли вона генерується з machine matrix і CI перевіряє byte/content drift.