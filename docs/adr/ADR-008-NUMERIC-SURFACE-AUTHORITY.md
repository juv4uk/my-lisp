# ADR-008 — Numeric surface authority

**Статус:** Accepted  
**Дата:** 2026-09-08

## Рішення

Етап A meaning-first міграції завершено.

`lib/surface/semantic-registry.wsm` є **єдиною машинною authority** для
відповідності між semantic identities та людськими програмними поверхнями.
Semantic identity у цьому реєстрі — непрозорий атом, що складається тільки з
цифр.

```text
                         0101
                           │
              ┌────────────┼────────────┐
              │            │            │
             UK           EN           SA
        відобразити       map       āvartana
```

Жодна людська назва не є ключем іншої людської назви.

## Спільна символіка

Пунктуаційна нотація не є людською мовою. Для неї існує окрема surface `sym`.

```text
                         0104
                ┌──────────┼──────────┐
                │          │          │
               UK         SA         sym
             додати      yoga         +

               EN: missing
```

Тому `+`, `-`, `*`, `/`, `<`, `>`, `=` та подібні знаки не можуть бути
зараховані EN лише тому, що історичний bootstrap використовував їх поруч з
англійськими словами. Якщо справжнє людське ім'я певної surface не
ратифіковане, реєстр чесно містить `— missing`.

## Legacy table

`lib/surface/uk-sa-coverage.wsm` зберігається як історичний аудит походження
імен і попередніх статусів. Вона **не є semantic authority** і не може живити:

- REPL introspection;
- surface parity gate;
- drift checker;
- program surface translator;
- документаційний join key.

Нові executable consumers мають читати тільки `semantic-registry.wsm`.

## REPL

`:ім'я` / `:name` показує numeric identity незалежно від того, яким spelling
виконано пошук.

```text
:ім'я map
:ім'я відобразити
:ім'я āvartana
:ім'я 0101
```

усі ведуть до:

```text
identity: 0101
```

Для shared notation:

```text
:ім'я +

identity: 0104
  UK: додати [stable]
  EN: — [missing]
  SA: yoga [stable]
  SYM: + [stable]
```

`(env)` / `(середовище)` залишається сирою lexical introspection і може
показувати історичні host/bootstrap spellings. Це не надає їм semantic
authority.

`core` також не є четвертою людською surface. У surface introspection він
показує numeric machine handles; його runtime environment може залишатися
implementation-oriented.

## Межа цього ADR

Цей ADR закриває **authority та introspection debt**. Він не стверджує, що всі
140+ public values уже мають direct runtime peer binding одного `Rc`/closure.
Це окремі етапи B–D плану повного рівноправ'я і потребують executable proof для
кожної мігрованої identity.

Отже не плутаємо:

```text
numeric authority complete        ✅
REPL numeric introspection         ✅
legacy EN key retired             ✅
all runtime peer bindings complete ⏳
full UK/EN/SA release parity       ⏳
```

## Supersession

Цей ADR завершує перехідний пункт ADR-007, за яким
`uk-sa-coverage.wsm` тимчасово залишалася authority під час міграції.

В ADR-005 зберігається фундаментальний висновок про peer human surfaces, але
його legacy-інтерпретація `canonical/EN` колонки більше не є чинною машинною
архітектурою.

> **Значення первинне. Мови рівноправні. Символи не є мовою.**
