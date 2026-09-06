# TCP host surface audit / Аудит host-межі TCP

Status: read-side semantic cut completed; write-side remains a separate audit target. / Статус: семантичний розріз читання завершено; сторона запису лишається окремою ціллю аудиту.

## English

The old `tcp-read` host capability combined two responsibilities:

1. **Host mechanism:** perform one socket read and observe the received bytes.
2. **Text semantics:** require those bytes to be valid UTF-8 and turn them into a Lisp string inside Rust.

That boundary has been split.

```text
TCP socket
  ↓
Rust: tcp-read-raw
  ↓
(byte ...)
  ↓
Lisp: utf8-decode-string
  ↓
Lisp: tcp-read-bytes->text
  ↓
Lisp: tcp-read
```

`tcp-read-raw` now performs one read of at most 64 KiB and returns the exact bytes as a proper list of exact integers `0..255`. EOF is the empty list. The host no longer runs `String::from_utf8` on received data.

`lib/tcp.my` owns the public text interpretation. Valid UTF-8 becomes the same string callers previously received; EOF remains the empty string. Invalid UTF-8 is preserved up to the language boundary and becomes explicit language data, currently `(rejected invalid-utf8)`, instead of a Rust-owned text-decoding error.

The strongest adversarial witness sends the single byte `255`: raw TCP must expose `(255)`, while public `tcp-read` must reject it in Lisp. An ownership test separately requires the host registry to contain `tcp-read-raw` but not `tcp-read`, and requires `tcp-read` to appear as a Lisp closure only after the TCP semantic layer loads.

During the integration migration, real socket/knowledge-exchange tests exposed a stack overflow in `unicode-scalars->string`. That was not hidden by enlarging worker stacks. The language implementation was corrected: Unicode scalar materialization is now tail-recursive, and a dedicated 2 MiB worker-thread test pins stack-safe materialization of a 1 KiB ASCII payload.

This is a **read-side HSS reduction**, not a claim that the entire TCP boundary is minimal. `tcp-write` still accepts a Lisp string and converts it to bytes in Rust with `as_bytes()`. Whether the write side should become `tcp-write-raw` plus language-owned UTF-8 encoding is the next independent question.

## Українська

Старий host capability `tcp-read` поєднував дві різні відповідальності:

1. **Host-механізм:** виконати одне читання із сокета й спостерігати отримані байти.
2. **Текстову семантику:** вимагати, щоб ці байти були валідним UTF-8, і перетворювати їх на Lisp-рядок усередині Rust.

Тепер цю межу розділено.

```text
TCP socket
  ↓
Rust: tcp-read-raw
  ↓
(byte ...)
  ↓
Lisp: utf8-decode-string
  ↓
Lisp: tcp-read-bytes->text
  ↓
Lisp: tcp-read
```

`tcp-read-raw` виконує одне читання максимум до 64 KiB і повертає точні байти як правильний список точних цілих `0..255`. EOF — порожній список. Host більше не запускає `String::from_utf8` над отриманими даними.

`lib/tcp.my` володіє публічною текстовою інтерпретацією. Валідний UTF-8 дає той самий рядок, який раніше отримували виклики; EOF лишається порожнім рядком. Невалідний UTF-8 доходить до мовної межі без втрати байтів і стає явними мовними даними — зараз `(rejected invalid-utf8)` — замість Rust-owned помилки декодування тексту.

Найсильніший adversarial witness передає один байт `255`: raw TCP мусить показати `(255)`, а публічний `tcp-read` мусить відхилити його вже в Lisp. Окремий ownership-тест вимагає, щоб host registry містив `tcp-read-raw`, але не `tcp-read`, а `tcp-read` з'являвся як Lisp closure лише після завантаження TCP semantic layer.

Під час міграції інтеграційні тести на реальних сокетах і knowledge exchange виявили stack overflow у `unicode-scalars->string`. Його не замасковано збільшенням стека worker thread. Виправлено саму мовну реалізацію: матеріалізація Unicode scalars тепер tail-recursive, а окремий тест із 2 MiB worker stack фіксує stack-safe матеріалізацію 1 KiB ASCII payload.

Це **зменшення HSS зі сторони читання**, а не твердження, що вся TCP-межа вже мінімальна. `tcp-write` досі приймає Lisp-рядок і перетворює його на байти в Rust через `as_bytes()`. Чи треба зробити сторону запису як `tcp-write-raw` + language-owned UTF-8 encoding — це наступне окреме питання.
