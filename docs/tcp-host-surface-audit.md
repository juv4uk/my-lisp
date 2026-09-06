# TCP host surface audit / Аудит host-межі TCP

Status: TCP text read/write semantics moved to Lisp; socket lifecycle remains a separate audit target. / Статус: текстову семантику читання/запису TCP перенесено в Lisp; життєвий цикл сокета лишається окремою ціллю аудиту.

## English

The old TCP host surface mixed socket transport with text interpretation. `tcp-read` decoded received bytes as UTF-8 in Rust, while `tcp-write` encoded a Lisp string through Rust `as_bytes()` before writing it.

Both text decisions are now outside the host.

```text
read:
TCP socket
  ↓
Rust: tcp-read-raw
  ↓ exact bytes 0..255
Lisp: utf8-decode-string
  ↓
Lisp: tcp-read

write:
Lisp string
  ↓
Lisp: utf8-encode-string
  ↓ exact bytes 0..255
Rust: tcp-write-raw
  ↓
TCP socket
```

`tcp-read-raw` performs one socket read of at most 64 KiB and returns a proper list of exact byte integers. EOF is the empty list. `lib/tcp.my` owns the public text interpretation: valid UTF-8 becomes a string, EOF becomes the empty string, and invalid UTF-8 becomes explicit language data such as `(rejected invalid-utf8)`.

`tcp-write-raw` accepts a proper list of exact byte integers `0..255` and writes exactly those bytes. It does not accept a string and contains no UTF-8 encoding rule. `lib/utf8.my` owns exact string-to-UTF-8 encoding, and `lib/tcp.my` composes that encoder with `tcp-write-raw` to provide the public `tcp-write` closure while preserving the historical result shape: a successful write returns the original text.

The minimal runtime bridges are deliberately below UTF-8 semantics: `codepoint->string` materializes one already-interpreted Unicode scalar, and `string->codepoint` observes the scalar value of exactly one runtime character. Neither bridge knows UTF-8. The byte encoding/decoding algorithms are Lisp code.

Evidence includes three adversarial/live witnesses: byte `255` is preserved by raw reads and rejected only by Lisp text decoding; raw writes transmit `(255 0 65 128)` byte-for-byte; and the public Lisp write path sends `"Привіт €😀"` as the exact UTF-8 wire bytes. Ownership tests require the host registry to contain `tcp-read-raw` and `tcp-write-raw`, but not `tcp-read` or `tcp-write`; after `load_tcp_library`, both public names must be Lisp closures. CI #959 passed workspace tests, build, and zero-warning clippy after the old Rust text `tcp-write` and its `as_bytes()` path were physically removed.

A separate migration bug also produced useful evidence: integration tests exposed stack growth while decoded Unicode scalars were materialized into text. The implementation was changed to tail-recursive accumulation rather than hiding the failure by increasing worker stack size, and a dedicated worker-thread test pins the behavior.

This is an evidenced **TCP text-semantics HSS reduction**, not a claim that the entire TCP boundary is minimal. `tcp-connect`, `tcp-listen`, `tcp-accept`, and `tcp-close` still represent host/socket operations. Their argument validation, error shaping, address policy, timeout behavior, retry policy, and lifecycle semantics remain independent audit targets.

## Українська

Стара TCP host-межа змішувала транспортування через сокет із текстовою інтерпретацією. `tcp-read` декодував отримані байти як UTF-8 у Rust, а `tcp-write` перед записом перетворював Lisp-рядок на байти через Rust `as_bytes()`.

Тепер обидва текстові рішення винесені з host-а.

```text
читання:
TCP socket
  ↓
Rust: tcp-read-raw
  ↓ точні байти 0..255
Lisp: utf8-decode-string
  ↓
Lisp: tcp-read

запис:
Lisp-рядок
  ↓
Lisp: utf8-encode-string
  ↓ точні байти 0..255
Rust: tcp-write-raw
  ↓
TCP socket
```

`tcp-read-raw` виконує одне читання із сокета максимум до 64 KiB і повертає правильний список точних цілих байтів. EOF — порожній список. `lib/tcp.my` володіє публічною текстовою інтерпретацією: валідний UTF-8 стає рядком, EOF — порожнім рядком, а невалідний UTF-8 — явними мовними даними на кшталт `(rejected invalid-utf8)`.

`tcp-write-raw` приймає правильний список точних цілих байтів `0..255` і записує саме ці байти. Він не приймає рядок і не містить правила UTF-8-кодування. `lib/utf8.my` володіє точним перетворенням рядка на UTF-8, а `lib/tcp.my` композиційно будує публічний Lisp closure `tcp-write` поверх `tcp-write-raw`, зберігаючи стару форму результату: успішний запис повертає початковий текст.

Мінімальні runtime-мости навмисно лежать нижче UTF-8-семантики: `codepoint->string` матеріалізує один уже інтерпретований Unicode scalar, а `string->codepoint` спостерігає scalar-значення рівно одного runtime-символу. Жоден із цих мостів не знає UTF-8. Самі алгоритми кодування й декодування байтів написані Lisp-ом.

Докази включають три adversarial/live witness-и: байт `255` зберігається raw-читанням і відхиляється лише Lisp-декодером; raw-запис передає `(255 0 65 128)` байт-у-байт; публічний Lisp-шлях запису передає `"Привіт €😀"` як точні UTF-8 байти на дроті. Ownership-тести вимагають, щоб host registry містив `tcp-read-raw` і `tcp-write-raw`, але не `tcp-read` чи `tcp-write`; після `load_tcp_library` обидві публічні назви мусять бути Lisp closure. CI #959 пройшов workspace tests, build і zero-warning clippy після фізичного видалення старого Rust `tcp-write` та його `as_bytes()`-шляху.

Під час цієї міграції інтеграційні тести також виявили ріст стека при матеріалізації декодованих Unicode scalar-ів у текст. Причину виправлено tail-recursive накопиченням, а не збільшенням стека worker thread; окремий тест тепер фіксує цю властивість.

Це доказане **зменшення TCP HSS для текстової семантики**, а не твердження, що вся TCP-межа вже мінімальна. `tcp-connect`, `tcp-listen`, `tcp-accept` і `tcp-close` лишаються host/socket операціями. Їхня валідація аргументів, форма помилок, адресна політика, timeout-и, retry-політика та lifecycle semantics — наступні незалежні цілі аудиту.
