# Перший вертикальний зріз Lisp → x86-64

**Дата:** 2026-09-14
**Статус:** виконано як експериментальний вертикальний доказ у PR #118
**Ціль:** довести збережений шлях, у якому my-lisp володіє семантичним lowering та формуванням x86-64 байтів, а host лишається семантично сліпим механізмом виконання.

## Що було заплановано

1. Відокремити три авторитети: семантика мови — `my-lisp`; факти ISA — специфікація процесора; оптимізація — CML.
2. Представити потрібні x86-64 факти як Lisp-дані без semantic ID.
3. Реалізувати Lisp-owned encoder для мінімального підмножини `MOV` / `ADD` / `RET`.
4. Додати вузький host-механізм виконання байтів з дисципліною W^X.
5. Під'єднати існуючу semantic identity `0104` до машинного fast path без перенесення значення операції в Rust.

## Реалізований стан

План виконано ширше, ніж початковий мінімум.

- `lib/machine/isa/` містить незалежні каталоги x86-64 та розширень, які декларує профіль Intel Core i5-6400.
- `lib/machine/cpu/intel-core-i5-6400.lisp` розділяє baseline, runtime-gated, platform-gated, virtualization та unavailable capabilities.
- `lib/machine/encoding/x86-64.lisp` у Lisp обчислює коди регістрів, REX, ModR/M, little-endian immediate та байти `MOV`, `ADD`, `RET`.
- `lib/machine/lowering/semantic-x86-64.lisp` дає bounded `u64` realization для semantic identity `0104`.
- proof lowering виправлено відповідно до SysV x86-64 ABI: використовується caller-saved `RCX`, а не callee-saved `RBX`.
- `crates/my-lisp-host/src/native_exec.rs` на Linux x86-64 робить лише `mmap RW → copy → mprotect RX → call → munmap`.
- `native-call-u64-raw` не має semantic ID і не вибирає інструкцію за Lisp-семантикою.

Фізичний witness:

```lisp
(native-call-u64-raw
  (x86-lower-add-u64 2 3))
```

Очікуваний і перевірений результат:

```text
5
```

## Межа твердження

Цей зріз **не** доводить повний native compiler або повну x86-64 реалізацію. Він доводить вужче:

```text
існуюча Lisp-семантика
        ↓
Lisp-owned semantic lowering
        ↓
Lisp-owned x86-64 encoding
        ↓
машинні байти
        ↓
семантично сліпий host-механізм
        ↓
фізичне виконання CPU
```

Зворотний напрямок заборонений: машинні факти або байти не можуть створювати, перевизначати чи присвоювати semantic IDs.

## Перевірки зрізу

Обов'язкові gates:

```text
cargo test -p my-lisp --test machine_lowering_boundary
cargo test -p my-lisp --test x86_64_lisp_encoder
cargo test -p my-lisp-host --test native_lisp_bytes
cargo clippy -p my-lisp -p my-lisp-host -p my-lisp-cli -p xtask --all-targets -- -D warnings
```

Окремий тест забороняє Rust native executor містити `0104`, `x86-lower`, `x86-encode`, `ADD` або `ADDSD` як семантичні рішення.

## Наступний доказ

Не розширювати ISA заради кількості. Наступний сильніший експеримент — interpreter/native parity для кількох значень і потім structural Lisp operation (`car` / `cdr`, далі `cons` після явного контракту representation/allocation).

Назва досягнення не повинна бути сильнішою за найсильніший експеримент, який його підтримує.
