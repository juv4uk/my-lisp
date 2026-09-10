#!/usr/bin/env python3
from pathlib import Path

path = Path("PLAN.md")
text = path.read_text(encoding="utf-8")

replacements = [
    (
        "> **Оновлено:** 2026-09-07.  ",
        "> **Оновлено:** 2026-09-10.  ",
        "roadmap date",
    ),
    (
        '''## A2. Meta-evaluator ownership

Підтверджені main-path slices:

- ✅ lexical closures;
- ✅ first-class builtins і lexical shadowing;
- ✅ macros;
- ✅ top-level `def`;
- ✅ self recursion;
- ✅ variadic і dotted lambda;
- ✅ finite mutual-recursion groups без cyclic host environment;
- ✅ unresolved callable `UnknownSymbol`;
- ✅ unresolved name vs non-callable `Type`;
- ✅ fixed/rest lambda `Arity`;
- ✅ malformed lambda-list `InvalidForm`.

Відомі межі:

- arbitrary later-binding visibility не доведена як загальна властивість;
- повна parity усіх native error classes не заявляється;
- `meta-eval` — explicit self-hosting witness, не always-loaded runtime.

Не виправляти later-binding через dynamic-scope shortcut. Сильніший proof має
лишатися lexical і finite-data.
''',
        '''## A2. Meta-evaluator ownership

Machine-readable evidence matrix `knowledge/meta-eval-evidence.wsm` є
авторитетом для parity-статусів; людська проєкція генерується з неї. На
2026-09-10 усі **34/34 required rows confirmed**.

Підтверджені main-path slices включають:

- ✅ lexical closures, nested capture і lexical shadowing;
- ✅ first-class builtins;
- ✅ macros і macro arity/error propagation;
- ✅ `def` / `define` top-level semantics;
- ✅ self recursion і dependency-aware finite mutual-recursion SCC;
- ✅ variadic і dotted lambda;
- ✅ shared top-level definition frame без dynamic scope;
- ✅ empty-program no-op semantics;
- ✅ application order і first-failure short-circuit;
- ✅ named error-kind correspondence та ратифікована diagnostic-detail boundary;
- ✅ registry-derived Canon / necessary-form surface identity у meta bootstrap.

Межі claim-а лишаються навмисно вужчими за список зелених тестів:

- `meta-eval` — explicit Lisp-owned self-hosting witness, не always-loaded runtime;
- 34/34 означає повноту **поточного required evidence scope**, а не доказ усіх
  можливих програм чи всіх майбутніх semantic extensions;
- `complete-self-hosting` не проголошується автоматично: силу такого claim-а
  окремо визначає claim vocabulary у machine evidence matrix.

Не повертати later-binding через dynamic-scope shortcut: чинний proof лишається
lexical і finite-data.
''',
        "A2 meta-evaluator block",
    ),
    (
        '''## C2. Arbitrary later-binding visibility

Explicit self-hosting proof gap. Не автоматичний bugfix backlog.

Потрібний proof має одночасно:

- бачити потрібні later top-level bindings;
- зберігати lexical scope;
- лишатися finite-data;
- не повертати cyclic mutable host environment.
''',
        '''## C2. Meta-evaluator claim discipline

Arbitrary later-binding visibility більше не є відкритим gap: shared definition
frame підтверджений paired evidence без dynamic scope і без cyclic mutable host
environment.

Наступну self-hosting роботу відкривати лише якщо вона:

- знаходить нову executable native/meta divergence поза чинними 34 required rows;
- потрібна Advice Taker або новому ратифікованому semantic extension;
- або формально змінює scope/силу self-hosting claim-а окремим рішенням.

Не розширювати matrix лише заради більшого числа тестів.
''',
        "C2 later-binding block",
    ),
]

for old, new, label in replacements:
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"guard failed for {label}: expected exactly 1 match, got {count}")
    text = text.replace(old, new, 1)

path.write_text(text, encoding="utf-8")
