# Межа self-hosting для `lib/meta-eval.my`

Цей документ є людською проєкцією машинно-читаної матриці `knowledge/meta-eval-evidence.wsm`. Він **не** створює нової семантичної влади: статуси мають відповідати executable evidence і перевіряються `scripts/check-meta-eval-evidence.py`.

## Дозволена лексика

- `metacircular evaluator exists` — дозволено: `my-eval` виконує репрезентативні my-lisp програми як Lisp-функція.
- `self-hosting witness` — дозволено лише для явно підтвердженого subset.
- `partial self-hosting` — дозволено, поки gaps названі й мають executable witnesses.
- `complete self-hosting` — **заборонено**, доки хоча б один required row має `partial`, `broken` або `unknown`.

## Поточна межа

Матриця навмисно не робить усі рядки зеленими. Найважливіші відкриті розбіжності:

| Рядок | Статус | Що доведено |
|---|---|---|
| `symbol-lookup-unknown-symbol` | `partial` | reference дає `UnknownSymbol`, meta зараз називає його `unbound-symbol`; bare unresolved symbol потребує окремого differential proof |
| `macro-arity-error` | `broken` | reference дає `Arity`, а meta повторно оцінює error-value як macro expansion і деградує до `unbound-symbol` |
| `adjacent-nonrecursive-defs-not-false-grouped` | `broken` | current recognizer збирає кожен contiguous lambda-def block у `recursive-group-closure`, навіть без recursive dependency |
| `arbitrary-later-binding-visibility` | `partial` | reference closure бачить пізніше визначення у shared frame; meta closure, захоплена до нього поза finite group, не бачить його |
| `function-application-order` | `unknown` | ще немає малого executable proof, де порядок спостережуваний без введення нової мутації |
| `define-form` | `unknown` | production evaluator має `0011 DEFINE`, але роль `define` у `my-eval-program` ще не доведена окремим paired experiment |
| `error-kind-parity` | `partial` | `Arity` та `InvalidForm` збігаються в аудитованих випадках; `UnknownSymbol`/`Type` vocabulary і macro arity ще розходяться |
| `error-detail-parity` | `partial` | kind є observable semantics; які саме detail fields мають бути parity-bound, ще треба формально відокремити від presentation text |

Водночас executable tests уже підтверджують Canon-first resolution, Canon binder rejection, QUOTE, COND, closures, fixed/variadic/dotted lambda binding, lexical capture, non-Canon shadowing, primitive bridge, arithmetic/chained comparisons, `def`, macro expansion, self recursion, 2- і 3-member mutual recursion, forward references усередині finite group, group-member shadowing, captured outer environment, nested closures та malformed-group `InvalidForm`.

## Правило інтерпретації

Differential mismatch — це **finding**, а не автоматично «meta-eval неправильний». Якщо reference runtime суперечить `language-contract.my` або ратифікованому ADR, під підозрою reference implementation. Матриця лише обмежує силу тверджень тим, що реально пережило executable comparison.
