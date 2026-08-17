# my-lisp для Racket / DrRacket

Платформенна підтримка мови **my-lisp** як повноцінного `#lang`-плагіна
для Racket. Код компілюється вбудованим JIT-компілятором Chez Scheme
(Racket CS, версія 8+).

> Важлива деталь діалекту: `quote` записується **явно** — `(quote x)` —
> без апострофа-скорочення `'x`. У my-lisp апостроф тепер є частиною
> символу (наприклад, `об'єкт`).

## Структура

```
racket/
├── info.rkt          ← опис пакета для raco + реєстрація мови в DrRacket
├── main.rkt          ← ядро: примітиви, truth-значення, exact /, defmacro, echo
├── reader.rkt        ← reader: S-вирази, ' як символ, exact decimal literals
├── lang/
│   └── reader.rkt    ← точка входу, яку шукає `#lang my-lisp`
└── README.md         ← ця інструкція
```

> Чому два reader'и? Рядок `#lang X` за контрактом шукає модуль
> `X/lang/reader`. Тому `lang/reader.rkt` — тонка обгортка, що
> реекспортує логіку з основного `reader.rkt`.

## Крок 1. Встановлення пакета

З кореня репозиторію виконайте в терміналі:

```sh
raco pkg install --link --name my-lisp racket/
```

(або з каталогу `racket/` — просто `raco pkg install --link`).
Прапорець `--link` не копіює файли, а створює посилання на каталог:
будь-які зміни у вихідниках одразу підхоплюються.

Перевірка, що колекцію зареєстровано:

```sh
raco pkg show my-lisp
```

## Крок 2. Запуск constitution.my у DrRacket

Приклад лежить у [`examples/constitution.my`](examples/constitution.my).

1. Додайте **першим рядком** цього файлу:

   ```lisp
   #lang my-lisp
   ```

2. Відкрийте файл у **DrRacket**. Ліворуч унизу має з'явитися напис
   «Determine language from source» / мова `my-lisp`
   (мову також можна обрати вручну: *Language → Choose Language… → my-lisp*).

3. Натисніть кнопку **Run** (Ctrl+R). DrRacket скомпілює модуль
   JIT-компілятором Chez Scheme й виконає його; у панелі взаємодій
   (REPL) одразу доступні всі визначення файлу.

Той самий файл можна запустити і з терміналу (з каталогу `racket/`):

```sh
racket examples/constitution.my
```

## Що дає мова

Сім примітивів Маккарті — `quote`, `atom` (`atom?`), `eq`, `car`, `cdr`,
`cons`, `cond`, — плюс:

```lisp
#lang my-lisp

;; Точна арифметика: ділення завжди повертає exact-дріб
(/ 5 336)          ; ⇒ 5/336 (не наближення!)
(/ 5.0 2)          ; ⇒ 5/2

;; Значення істини
(if t 'yes 'no)    ; ⇒ yes
(if () 'yes 'no)   ; ⇒ no

;; Класичні макроси
(defmacro (when test . body)
  `(if ,test (begin ,@body)))

(when (atom (quote x))
  (print "x — атом"))
```

## Echo-політика REPL

Як і в вашому CLI, у DrRacket (вкладка взаємодій / REPL) введення
одиночного нев'язаного ідентифікатора друкує `echo <ідентифікатор>`
замість помилки:

```text
> hello
echo hello
> мама
echo мама
> (+ 1 2)
3
```

## Відомі обмеження

* `defmacro` реалізований як **compile-time** макрос (через
  Racket `define-syntax`). Це достатньо для простих макросів
  (`when`, `unless`, одноразові генератори коду), але макроси
  bootstrap-бібліотеки `lib/core.my`, які в тілі викликають інші
  користувацькі визначення (наприклад, `(defmacro let ...)`
  використовує `second` та `map`), не розгортаються — їх потрібно
  або вбудувати в `main.rkt`, або реалізовувати через повноцінний
  runtime-розгортувач макросів.
* Повна bootstrap-бібліотека (`lib/core.my`, `lib/reason.my` тощо)
  ще не портована; плагін покриває Level 1/2 семантичного контракту
  (core primitives + мова/арифметика).

## Видалення пакета

```sh
raco pkg remove my-lisp
```
