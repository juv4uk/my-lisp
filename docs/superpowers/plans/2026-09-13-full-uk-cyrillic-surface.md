# Історичний план повної української Cyrillic surface

> **Статус: виконаний/перевершений історичний план. Не запускати як активний
> implementation checklist.**
>
> План збережено як design capital зі старої `feat/full-uk-cyrillic-surface`.
> Початкове ім'я `full-uk` більше не є чинним namespace: сучасний `main`
> використовує `ukr` як повну українську peer surface. Поточна authority —
> `lib/surface/semantic-registry.lisp`.

## Початкова мета

Історична робота мала довести не просто наявність перекладених слів, а повну
вертикаль:

```text
numeric semantic ID
        ↓
повне українське spelling
        ↓
registry admission
        ↓
generated projections
        ↓
native/meta runtime
        ↓
реальна програма без перемикання на Latin layout
```

Важливо, що навіть первинний план не вимагав окремої української реалізації:
numeric semantic ID мав лишатися єдиною identity.

## Як змінилася архітектура під час виконання

Початково план використовував назву `full-uk` поряд із `uk`. У ході роботи
з'ясувалося, що вже існуючий label `ukr` може виконувати роль повної
української поверхні без третьої української колонки.

Еволюція:

```text
uk + full-uk + допоміжна ukr projection
                ↓
усунення дублювання
                ↓
uk + ukr
```

Тому нижче збережено **намір кожної фази**, а не старі команди й шляхи як
нормативну інструкцію.

## Фаза 1 — зробити no-layout-switch rule виконуваним

Історична задача: автоматично знаходити candidate spellings із ASCII Latin
letters та не дозволяти їм непомітно стати повною українською surface.

Проблемні приклади того часу:

```text
прочитати-з-tcp
розібрати-json
sha256-у-шістнадцятковий-текст
```

Урок, який лишається чинним: **перевіряти властивість, а не список вручну**.
Нове ім'я або новий semantic ID не повинні обходити guard лише тому, що їх не
додали до hard-coded audit table.

## Фаза 2 — зробити повну українську peer surface частиною registry

Початковий план хотів додати authoritative `full-uk` triples. Реалізація
еволюціонувала: роль перейшла до `ukr`.

Чинна форма принципу:

```text
(semantic-id
  ...
  (uk  COMPACT-SPELLING status)
  (ukr FULL-SPELLING    status)
  ...)
```

Registry parser та consumers мають бути namespace-generic; окремий evaluator
для української поверхні не потрібен.

## Фаза 3 — генератори не мають вигадувати українські назви

Старий план окремо атакував небезпечну модель, де generated function table
могла синтезувати `full-uk` шляхом копіювання `uk`.

Збережений принцип:

```text
registry = authority
       ↓
generator = projection only
```

Якщо spelling відсутній або має candidate status, generator повинен показати
цей факт, а не домислити «правильну» назву.

Пізніший refactor `ae30e087df3e8c8d1a9a213ab47301c3731c0d78` прибрав дубльовану
`full-uk` колонку й зробив `ukr` прямою повною українською projection.

## Фаза 4 — native і meta повинні бачити одну identity

План вимагав executable witness: різні peer spellings повинні матеріалізувати
одну semantic identity, а не просто випадково викликати схожу реалізацію.

Цей принцип пізніше став ще сильнішим у загальній Canon/SemanticRef роботі:
semantic identity не повинна залежати від Rust pointer/Builtin identity.

## Фаза 5 — реальна Cyrillic-only acceptance program

Кінцевий доказ задумувався як програма, де executable identifiers можна набрати
в українській розкладці без стрибків до Latin.

Acceptance мав перевіряти не один toy-call, а композицію:

- визначення;
- функцію;
- умовний вибір;
- арифметику;
- list processing;
- higher-order operation;
- не-core capability.

Ця вимога важлива як метод: surface вважається реальною лише тоді, коли нею
можна написати програму, а не коли таблиця coverage показує 100%.

## Фаза 6 — повна verification перед твердженням «готово»

Історичний план вимагав перевіряти:

- відсутність drift semantic IDs;
- збереження existing `uk` compatibility;
- generated projections проти authority;
- no-Latin invariant для повної української surface;
- runtime acceptance;
- repository policy gates.

Цей підхід пережив конкретну naming-роботу і став загальнішим правилом проєкту:
статус або зелений одиночний тест не замінює exact evidence.

## Старі шляхи та сучасні відповідники

Під час історичної роботи репозиторій ще використовував старі розширення й
назви. Для читача важливо не сприймати їх буквально:

| Історичне посилання | Чинний сенс |
|---|---|
| `lib/surface/semantic-registry.wsm` | `lib/surface/semantic-registry.lisp` |
| namespace `full-uk` | повна українська peer surface `ukr` |
| synthesized `ukr` table view | прямий `ukr` registry namespace |
| generated table як доказ naming | лише projection; authority лишається registry |

Інші конкретні файли/команди зі старого плану могли бути перейменовані або
видалені; їх потрібно шукати в сучасному `main`, а не replay-ити механічно.

## Що свідомо НЕ треба replay-ити

Не переносити зі старої гілки:

- її registry/code delta wholesale;
- one-shot migration scripts;
- generated artifacts як джерело істини;
- старий `full-uk` namespace;
- acceptance tests, якщо сучасні тести вже доводять ту саму властивість іншою
  архітектурою;
- старі candidate names лише тому, що вони колись були в плані.

## Що варто зберегти для майбутніх агентів

1. **Surface UX є технічною властивістю.** «Без перемикання розкладки» можна і
   треба доводити executable guard-ом.
2. **Generator не ратифікує.** Він лише проєктує authority.
3. **Coverage не сильніший за naming quality.** Краще `candidate/missing`, ніж
   погане stable spelling.
4. **Peer surface — не implementation fork.** Усі мови сходяться в numeric
   semantic identity.
5. **Acceptance має бути програмою.** Таблиці й grep-и допоміжні, але не
   замінюють виконання.
6. **Старий план — evidence, не команда.** Перед replay завжди звіряти current
   `main`.

## Історичні джерела

Основний plan commit:

- `e426cd1158dc7cdec948a8343136fc7bfdc285c7` —
  `docs(uk): plan full Ukrainian authority ratification`.

Ключові подальші зміни, які змінили фінальну форму плану:

- `1e15e2b5d944054e50fc379129c334b1c04011ea` — `ukr` визначено повною
  українською surface;
- `ae30e087df3e8c8d1a9a213ab47301c3731c0d78` — усунено дубльовану
  `full-uk` projection;
- `9f891091e4d97b1443232808e4fe8d707fd95706` — документація `uk`/`ukr`
  пояснена як compact/full pair.

Цей документ закриває історичний контекст, але не відкриває нової naming
міграції.