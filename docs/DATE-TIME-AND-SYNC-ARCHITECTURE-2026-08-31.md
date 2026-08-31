# Date, time, timezone, and synchronization / Дата, час, timezone і синхронізація

This document is the practical map of WSM's current time-related interfaces.
It distinguishes elapsed time, calendar time, external time observations,
timezone configuration, logical filesystem history, and Guard synchronization
policy. It does not claim an OS clock setter or a complete timezone database.

Це практична карта поточних часових інтерфейсів WSM. Вона розділяє тривалість,
календарний час, зовнішнє спостереження часу, конфігурацію timezone, логічну
історію файлової системи й policy синхронізації Guard. Встановлення системного
годинника та повна база часових поясів наразі не заявляються.

## Interfaces / Інтерфейси

| Interface | Meaning | Use it for |
|---|---|---|
| `(mono-ms)` | monotonic milliseconds since process start | coarse durations |
| `(mono-ns)` | exact monotonic nanoseconds since process start | precise durations/timeouts |
| `(utc-now)` | `(utc year month day hour minute second nanosecond)` | UTC observation receipts |
| `(internet-time-sync host timeout-ms)` | bounded NTP observation | compare with an external clock |
| `(timezone-detect)` | explicit `TZ` or `/etc/timezone` observation | host timezone discovery |
| `(timezone-config name offset)` | ordinary immutable WSM data | explicit chosen timezone config |

| Інтерфейс | Значення | Для чого |
|---|---|---|
| `(mono-ms)` | монотонні мілісекунди від старту процесу | грубі тривалості |
| `(mono-ns)` | точні монотонні наносекунди від старту процесу | точні тривалості/таймаути |
| `(utc-now)` | `(utc рік місяць день година хвилина секунда наносекунда)` | UTC-мітки спостережень |
| `(internet-time-sync host timeout-ms)` | обмежене NTP-спостереження | порівняння із зовнішнім годинником |
| `(timezone-detect)` | спостереження явного `TZ` або `/etc/timezone` | визначення timezone хоста |
| `(timezone-config name offset)` | звичайні immutable WSM-дані | явна конфігурація timezone |

## Do not collapse distinctions / Не зливайте різні поняття

```text
mono-ns       != utc-now
wall clock    != logical revision
timezone name != fixed offset rules
NTP result    != authenticated truth
timestamp     != content identity
```

`mono-ns` is not a date and can reset when the process restarts. `utc-now` is
calendar time and can move backward if the host clock is corrected. NTP returns
an observation and currently does not set the host clock. A nanosecond field is
represented exactly, but the physical clock may have coarser resolution.

`mono-ns` — не дата і може початися заново після перезапуску процесу. `utc-now`
— календарний час і може рухатися назад після корекції годинника хоста. NTP
повертає спостереження і наразі не встановлює системний годинник. Поле
наносекунд представляється точно, але фізичний годинник може мати грубшу
роздільність.

## Filesystem and evidence boundaries / Межі FS та evidence

WSM-FS uses immutable roots, journal order, logical revisions, and
content-addressed values. It must not use wall-clock timestamps for identity or
ordering. A timestamp may be attached to an operational receipt or observation,
but the filesystem claim is proved by root/journal evidence.

WSM-FS використовує immutable roots, порядок journal, логічні revisions і
content-addressed values. Wall-clock не використовується для identity чи
ordering. Часова мітка може бути metadata operational receipt або observation,
але filesystem claim доводиться root/journal evidence.

## Guard synchronization policy / Policy синхронізації Guard

Before an ecosystem synchronization, Guard requires:

```text
freeze commit/push activity
  → synchronize
  → inspect build and CI logs
  → record observed drift (including none observed)
  → reopen commit window
```

Перед ecosystem sync Guard вимагає:

```text
заморозити commit/push
  → виконати sync
  → переглянути build і CI logs
  → записати observed drift (у тому числі «не виявлено»)
  → відкрити вікно комітів
```

This is an explainable policy gate, not a hidden Git lock. `REJECT` means the
freeze was not observed; `WARN` means sync or drift recording is incomplete;
`ALLOW` means the complete sequence has evidence. See the ecosystem document
`ecosystem/docs/GUARD-SYNC-COMMIT-FREEZE.md` and `guard-sync-window` in
`lib/guard.wsm`.

Це пояснювана policy gate, а не прихований Git lock. `REJECT` означає, що
freeze не підтверджений; `WARN` — sync або запис drift неповні; `ALLOW` — для
всієї послідовності є evidence. Див. bilingual policy-документ і
`guard-sync-window` у `lib/guard.wsm`.
