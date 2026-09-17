# План реалізації генератора meta semantic registry

**Мета:** замінити активний Python-генератор projection нативною реалізацією на my-lisp, не змінюючи межу authority та значення згенерованої проєкції.

**Дизайн:** `docs/superpowers/specs/2026-09-17-python-to-lisp-meta-registry-design.md`.

1. **RED gate:** змінити focused projection CI так, щоб він збирав `my-lisp` і викликав `scripts/generate-meta-semantic-registry.lisp --check`. Підтвердити, що PR падає саме через відсутність нового скрипта.
2. **GREEN generator:** додати `scripts/generate-meta-semantic-registry.lisp`, структурно читати `lib/surface/semantic-registry.lisp`, нормалізувати reader-only апостроф, допускати лише `stable`/`compatibility-only` surfaces, дедуплікувати однакові spelling тієї самої identity, fail-closed відхиляти колізію між різними semantic IDs і детерміновано рендерити результат.
3. **Parity:** регенерувати `lib/generated/meta-semantic-registry.lisp`; єдина навмисна текстова зміна provenance після cutover — `.py` → `.lisp` у рядку generator. Прогнати новий `--check` gate.
4. **Negative witness:** додати режим `--self-test` для same-ID dedupe, cross-ID collision, reader-only apostrophe exclusion, candidate/missing exclusion і детермінованого sample rendering. Підключити його до focused CI.
5. **Cutover:** замінити всі активні CI-виклики Python-generator, оновити path classification та живі documentation/test references, після чого видалити `scripts/generate-meta-semantic-registry.py`.
6. **Перевірка:** оглянути точний PR diff, прогнати PR workflows, переконатися, що активних посилань на `generate-meta-semantic-registry.py` не лишилося поза immutable/historical evidence, і що інші Python-скрипти не зачеплені — вони підуть окремими зрізами #76.
