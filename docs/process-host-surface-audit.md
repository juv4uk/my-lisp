# Process host surface audit / Аудит host-межі процесів

Status: architectural audit, no host cut yet. / Статус: архітектурний аудит, host-розріз ще не виконано.

## English

`process-run` currently combines two different responsibilities:

1. **Host mechanism:** start a named program without a shell, pass an explicit argument vector, wait for completion, observe exit status/stdout/stderr.
2. **Interpretation:** convert stdout and stderr byte sequences into Lisp strings with `String::from_utf8_lossy`.

The first responsibility is genuinely host-owned. The second is semantic policy: invalid UTF-8 is silently replaced with U+FFFD, so byte identity is lost before Lisp can decide how to interpret the process output.

Current shape:

```text
OS process
  ↓
Rust Command::new(program).args(args).output()
  ↓
exit code + stdout bytes + stderr bytes
  ↓
Rust String::from_utf8_lossy
  ↓
(exit-code stdout-string stderr-string)
```

The repository already proves that raw byte transport is useful and representable: `read-file-bytes` / `write-file-bytes` expose byte lists without forcing UTF-8 interpretation. That makes a byte-preserving process boundary plausible, but it does **not** yet prove that Lisp has the inverse text-decoding substrate needed to preserve the current convenient public API.

Target architecture, only after the missing language substrate exists:

```text
Rust: process-run-raw
  ↓
(process-result exit-code stdout-bytes stderr-bytes)
  ↓
Lisp: explicit UTF-8 decode / policy
  ↓
process-run
```

Do not remove or rename the existing `process-run` yet. First add a language-visible byte↔text conversion contract with deterministic tests, then add the raw host capability, then make the public text-oriented `process-run` language-owned, then migrate consumers, and only then remove the semantic duplicate from Rust.

### Findings

- `Command::new(...).args(...)` is mechanism and should remain host-owned.
- The per-session allowlist is a capability/security boundary, not ordinary language semantics; it should remain attached to the host capability unless a stronger capability-object model replaces it.
- Exit-status observation is host data. Mapping “no numeric exit code” to `-1` is a small policy choice and should be audited separately rather than silently treated as irreducible.
- `String::from_utf8_lossy` is the clearest current semantic leak because it can destroy information.
- TCP is related but different: `tcp-read` currently rejects invalid UTF-8 instead of replacing bytes. A future byte-oriented network capability may be worthwhile, but this audit does not claim it is required for the process cut.

### Evidence gate before cutting

The process HSS reduction counts only when all of these are true:

```text
byte-preserving raw process result exists
+ deterministic byte/text interpretation tests exist
+ public process-run is a Lisp closure
+ existing consumers pass unchanged or are explicitly migrated
+ lossy conversion is absent from the raw host path
+ CI is green
```

Until then this is a **identified semantic leak**, not a completed reduction.

## Українська

`process-run` зараз змішує дві різні відповідальності:

1. **Host-механізм:** запустити названу програму без shell, передати явний список аргументів, дочекатися завершення та спостерігати exit status/stdout/stderr.
2. **Інтерпретацію:** перетворити байти stdout і stderr на Lisp-рядки через `String::from_utf8_lossy`.

Перша відповідальність справді належить host-у. Друга — це семантична політика: невалідний UTF-8 мовчки замінюється символом U+FFFD, тому тотожність початкових байтів втрачається ще до того, як Lisp може вирішити, що ці байти означають.

Поточна форма:

```text
OS process
  ↓
Rust Command::new(program).args(args).output()
  ↓
exit code + stdout bytes + stderr bytes
  ↓
Rust String::from_utf8_lossy
  ↓
(exit-code stdout-string stderr-string)
```

У репозиторії вже є доказ, що сирі байти корисні й представимі: `read-file-bytes` / `write-file-bytes` передають списки байтів без примусової UTF-8-інтерпретації. Це робить byte-preserving межу процесів реалістичною, але ще **не доводить**, що Lisp має достатній зворотний byte→text substrate, щоб зберегти зручний поточний API.

Цільова архітектура — лише після появи потрібного мовного substrate:

```text
Rust: process-run-raw
  ↓
(process-result exit-code stdout-bytes stderr-bytes)
  ↓
Lisp: явне UTF-8 decode / policy
  ↓
process-run
```

Поточний `process-run` поки не видаляємо і не перейменовуємо. Спочатку потрібен мовний контракт byte↔text з детермінованими тестами, потім сирий host capability, потім language-owned публічний `process-run`, міграція споживачів, і лише після цього — ампутація семантичного дубля з Rust.

### Висновки

- `Command::new(...).args(...)` — механізм і має залишитися в host.
- Session allowlist — capability/security boundary, а не звичайна семантика мови; його варто залишити біля host capability, доки не з'явиться сильніша модель capability-об'єктів.
- Exit status — host-спостереження. Правило “немає числового exit code → `-1`” є окремою маленькою політикою і заслуговує власного аудиту.
- `String::from_utf8_lossy` — найчіткіший поточний semantic leak, бо він може безповоротно втратити інформацію.
- TCP споріднений, але не тотожний випадок: `tcp-read` зараз відхиляє невалідний UTF-8, а не замінює байти. Можлива майбутня byte-oriented network boundary, але цей аудит не оголошує її необхідною для process-розрізу.

### Evidence gate перед розрізом

Зменшення Process HSS рахується тільки якщо одночасно виконано:

```text
є byte-preserving raw process result
+ є детерміновані byte/text тести
+ public process-run є Lisp closure
+ споживачі мігровані або працюють без змін
+ у raw host path немає lossy conversion
+ CI зелений
```

До того моменту це **виявлений semantic leak**, а не завершене HSS-зменшення.
