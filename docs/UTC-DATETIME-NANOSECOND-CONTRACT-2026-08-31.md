# UTC date-time primitive / Примітив дати й часу UTC

Status: `CONFIRMED experimental` — implemented in the Rust kernel and covered
by `crates/my-lisp/tests/clock.rs`.

Статус: `CONFIRMED experimental` — реалізовано в ядрі Rust і перевірено
тестом `crates/my-lisp/tests/clock.rs`.

## Contract

`(utc-now)` is a zero-argument host observation. It returns the data-only
shape:

```lisp
(utc year month day hour minute second nanosecond)
```

`year`, `month`, `day`, `hour`, `minute`, `second`, and `nanosecond` are exact
integers. The calendar is proleptic Gregorian UTC; `nanosecond` is in
`0..999999999`. The primitive observes the wall clock and is not monotonic.

`(utc-now)` — це host-спостереження без аргументів. Воно повертає data-only
форму:

```lisp
(utc рік місяць день година хвилина секунда наносекунда)
```

Усі поля — точні цілі числа. Календар — пролептичний григоріанський UTC,
`nanosecond` лежить у `0..999999999`. Примітив читає настінний годинник і не є
монотонним таймером.

## Boundaries

- Use `mono-ns` for elapsed durations and benchmarks; its nanosecond integer
  representation is exact, but hardware clock resolution may be coarser.
- Use `utc-now` for timestamping observations or operational receipts.
- Do not use wall-clock time as content identity, logical FS revision, or
  distributed ordering.
- No timezone database, locale formatting, or date parsing is claimed yet.

`(internet-time-sync host timeout-ms)` performs one bounded NTP query and
returns either `(accepted host unix-seconds nanosecond)` or `(rejected reason)`.
It never sets the operating-system clock. `(timezone-detect)` reports an
explicit `TZ` or `/etc/timezone` declaration, or returns `unknown`; it does not
guess a timezone. A caller may keep a chosen fixed-offset timezone as ordinary
WSM data, but timezone rules and system mutation remain outside this contract.

- Для тривалостей і benchmark-ів використовуйте `mono-ns`.
- Для часових міток спостережень чи operational receipts використовуйте
  `utc-now`.
- Не використовуйте wall-clock як content identity, логічну revision FS або
  порядок у розподіленій системі.
- База часових поясів, локалізоване форматування і parsing дат поки не заявлені.
- `(internet-time-sync host timeout-ms)` робить один обмежений NTP-запит і
  повертає `(accepted host unix-seconds nanosecond)` або `(rejected reason)`.
  Системний годинник він не встановлює. `(timezone-detect)` читає явну
  декларацію `TZ` або `/etc/timezone`, а за її відсутності повертає `unknown`;
  часовий пояс не вгадується. Обраний фіксований offset можна зберігати як
  звичайні WSM-дані, але правила часових поясів і мутація ОС не входять у цей
  контракт.
