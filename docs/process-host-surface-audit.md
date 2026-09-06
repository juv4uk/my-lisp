# Process host surface audit / Аудит host-межі процесів

Status: **completed HSS reduction**. The semantic `process-run` host duplicate has been physically removed; `process-run-raw` is the remaining OS mechanism. CI #933 passed `cargo test --workspace`, `cargo build --workspace`, and zero-warning clippy after consumer migration.

Статус: **завершене зменшення HSS**. Семантичний host-дублікат `process-run` фізично видалено; `process-run-raw` лишився як механізм ОС. CI #933 пройшов `cargo test --workspace`, `cargo build --workspace` і clippy без попереджень після міграції споживачів.

## English

The old host path mixed two responsibilities:

1. **Mechanism:** start a named program without a shell, pass an explicit argument vector, wait for completion, and capture exit status/stdout/stderr.
2. **Meaning:** decode stdout/stderr with `String::from_utf8_lossy`, replace invalid bytes, and map a missing numeric exit status to `-1`.

The cut now has this shape:

```text
OS process
  ↓
Rust: process-run-raw
  ↓
(process-result exit-code-or-() stdout-bytes stderr-bytes)
  ↓
Lisp: utf8-decode-string + process-result->text
  ↓
Lisp: process-run
  ↓
(exit-code stdout-string stderr-string)
```

`process-run-raw` owns process creation, the per-session allowlist, exit observation, and byte capture. It preserves stdout/stderr byte identity and performs no text decoding. `lib/utf8.my` validates/decodes UTF-8, while `lib/process.my` owns rejection policy, public result shaping, and the historical “no numeric exit code → -1” convention.

Invalid UTF-8 is no longer silently replaced with U+FFFD. The public Lisp layer returns an explicit rejection such as:

```lisp
(rejected stdout-invalid-utf8)
```

Valid UTF-8 preserves the compatibility shape:

```lisp
(exit-code stdout-string stderr-string)
```

### Evidence

The reduction satisfies the original gate:

```text
byte-preserving raw process result                     ✓
deterministic UTF-8 interpretation tests               ✓
public process-run is a Lisp closure                   ✓
CLI / TCP / oracle / Yantra consumers migrated         ✓
legacy String::from_utf8_lossy process path removed    ✓
legacy host process-run registration removed           ✓
cargo test --workspace                                 ✓
cargo build --workspace                                ✓
clippy -D warnings                                     ✓
```

CI #933 is the first full green workspace run after the physical host removal and the Yantra/host-test migration. The black-box CLI tests also distinguish the new path from the old one: a child process emitting byte `255` yields the Lisp rejection value rather than a lossy replacement character.

The allowlist remains a host capability/security boundary. Moving UTF-8 and result semantics to Lisp does not imply that language code should be able to bypass embedding restrictions.

TCP is a separate audit. `tcp-read` still interprets network bytes as UTF-8 in Rust; this process reduction does not claim that network boundary has already been minimized.

## Українська

Старий host-шлях змішував дві відповідальності:

1. **Механізм:** запустити названу програму без shell, передати явний список аргументів, дочекатися завершення та отримати exit status/stdout/stderr.
2. **Значення:** декодувати stdout/stderr через `String::from_utf8_lossy`, замінювати невалідні байти та перетворювати відсутній числовий exit status на `-1`.

Після розрізу межа така:

```text
процес ОС
  ↓
Rust: process-run-raw
  ↓
(process-result exit-code-or-() stdout-bytes stderr-bytes)
  ↓
Lisp: utf8-decode-string + process-result->text
  ↓
Lisp: process-run
  ↓
(exit-code stdout-string stderr-string)
```

`process-run-raw` володіє лише створенням процесу, session allowlist, спостереженням exit status і захопленням байтів. Він зберігає тотожність stdout/stderr і не вирішує, що ці байти означають як текст. `lib/utf8.my` виконує валідацію/декодування UTF-8, а `lib/process.my` володіє політикою відхилення, формою публічного результату та історичним правилом “немає числового exit code → `-1`”.

Невалідний UTF-8 більше не перетворюється мовчки на U+FFFD. Lisp повертає явне значення, наприклад:

```lisp
(rejected stdout-invalid-utf8)
```

Для валідного UTF-8 збережено сумісну форму:

```lisp
(exit-code stdout-string stderr-string)
```

### Доказ

Початковий evidence gate тепер виконаний повністю:

```text
byte-preserving raw process result                     ✓
детерміновані тести UTF-8                              ✓
public process-run є Lisp closure                      ✓
CLI / TCP / oracle / Yantra мігровані                  ✓
legacy lossy process path фізично видалений            ✓
legacy host registration process-run видалений         ✓
cargo test --workspace                                 ✓
cargo build --workspace                                ✓
clippy -D warnings                                     ✓
```

CI #933 — перший повний зелений workspace-run після фізичного видалення host-дубліката та міграції Yantra/host-тестів. Black-box CLI-тест додатково відрізняє нову архітектуру від старої: процес, який видає байт `255`, дає Lisp-відхилення, а не символ заміни від lossy-декодування.

Allowlist лишається host-межею capability/security. Те, що семантика UTF-8 і форми результату повернулася Lisp-у, не означає обходу embedding-обмежень.

TCP — окремий аудит. `tcp-read` досі інтерпретує мережеві байти як UTF-8 у Rust; це зменшення Process HSS не оголошує мережеву межу вже мінімізованою.
