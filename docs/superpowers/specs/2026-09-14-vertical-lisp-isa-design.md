# Архітектура вертикального Lisp → ISA

**Дата:** 2026-09-14  
**Статус:** перший вертикальний зріз реалізований у PR #118; архітектура лишається інкрементальною  
**Область:** машинна межа `my-lisp`, Intel Core i5-6400 (Skylake), x86-64

## Мета

`my-lisp` має залишатися єдиним авторитетом щодо значення мовних операцій і stable semantic identities, але поступово володіти дедалі нижчими шарами їх фізичної реалізації.

Цільова вертикаль:

```text
semantic registry
      ↓
semantic lowering у Lisp
      ↓
ISA facts у Lisp
      ↓
Lisp-owned encoder
      ↓
машинні байти
      ↓
вузький host execution mechanism
      ↓
фізичний CPU
```

Rust не вилучається насильно. Він допускається там, де ОС контролює механізм: executable memory, page protection, ABI call, release memory. Але Rust не повинен знати значення semantic ID або вибирати машинну інструкцію замість Lisp.

## Модель авторитетів

Є три незалежні авторитети.

### 1. Семантичний авторитет

`lib/surface/semantic-registry.lisp` володіє мовним значенням і semantic IDs.

Приклад:

```text
0104 = існуюча semantic identity операції додавання
```

Машинні факти не можуть створювати або перевизначати semantic IDs.

### 2. ISA-авторитет

Фізичний набір інструкцій є зовнішнім фактом процесора. Для Intel x86-64 джерелами є Intel SDM / Intel XED та SKU-факти конкретного CPU.

Репозиторій може нормалізувати ці факти як Lisp-дані, але існування `ADD`, `AESENC`, `VPMADDUBSW` чи інших інструкцій не походить із language contract.

### 3. Авторитет оптимізації

CML може вибирати instruction sequences, scheduling, register allocation, vectorization та target-specific fast paths. CML не володіє існуванням чи ідентичністю машинних інструкцій і не замінює semantic registry.

## Односпрямоване правило

```text
machine fact → semantic ID     ЗАБОРОНЕНО
semantic meaning → machine use ДОЗВОЛЕНО
```

Це правило є важливішим за конкретний encoder або backend.

## Структура репозиторію

Поточний напрямок:

```text
lib/machine/
├── isa/          ; факти x86-64 та extension families
├── cpu/          ; конкретний CPU profile
├── encoding/     ; Lisp-owned physical encoding
└── lowering/     ; projection semantic meaning → machine realization
```

`lib/machine/**` не є новою public Lisp surface. Це внутрішній машинний шар.

## CPU profile Intel Core i5-6400

Профіль конкретного target має бути перетином:

```text
Intel/Skylake ISA facts
      ∩
Core i5-6400 SKU capabilities
      ∩
runtime CPUID / XGETBV state
      ∩
platform / firmware availability
      ∩
user-mode legality
```

Тому capability profile розділяє:

- baseline instruction families;
- runtime-gated extensions;
- platform-gated features;
- virtualization capabilities;
- explicitly unavailable capabilities.

`VT-x`, `VT-d`, `EPT` не треба змішувати зі звичайним user-mode semantic lowering. `TSX`, `AVX-512`, `AMX` для цього target не рекламуються як доступні.

## Lisp-owned encoder

Encoder має бути звичайним Lisp-кодом і Lisp-даними всюди, де це практично.

Перший proof subset:

```text
MOV
ADD
RET
```

Він уже обчислює:

- register codes;
- REX prefix;
- ModR/M;
- little-endian immediates;
- готові byte lists.

Приклад:

```text
ADD rax, rcx → 48 01 C8
RET          → C3
```

Зовнішній textual assembler не входить до retained proof path.

## Host/bootstrap boundary

Host має право:

- отримати byte list;
- виділити writable memory;
- скопіювати байти;
- змінити protection на executable;
- викликати адресу за визначеним ABI;
- звільнити memory.

Host не має права:

- знати, що `0104` означає додавання;
- вибирати `ADD` за semantic ID;
- створювати semantic IDs;
- декодувати машинні байти назад у мовну семантику.

Для Linux x86-64 proof використовується W^X:

```text
mmap RW → copy → mprotect RX → call → munmap
```

Ніколи не потрібна постійна RWX mapping.

## Семантичний lowering

Машинна реалізація є лише bounded realization існуючої семантики.

Приклад:

```text
0104
 ↓
bounded u64 path
 ↓
MOV / ADD / RET
 ↓
Lisp encoder
 ↓
bytes
```

Це **не** означає, що `ADD` є значенням semantic identity `0104`. Для arbitrary precision, rationals чи іншого representation semantic contract лишається незмінним і може використовувати інший Lisp/runtime path.

## Реалізований фізичний witness

```lisp
(native-call-u64-raw
  (x86-lower-add-u64 2 3))
```

Результат:

```text
5
```

Байти вибираються й формуються у Lisp. Host лише виконує їх.

Proof також перевіряє ABI discipline: тимчасовий регістр у викликаній SysV x86-64 функції — caller-saved `RCX`, а не callee-saved `RBX`.

## Портативність

Машинні факти та encoder можуть бути portable як дані й Lisp-код навіть там, де native execution недоступне.

Поточний `native-call-u64-raw` зареєстрований лише на:

```text
Linux + x86_64
```

Це не ламає Windows build: модуль і capability cfg-gated. Але Win64 native execution ще потребує окремого механізму на основі `VirtualAlloc` / `VirtualProtect` з тим самим W^X принципом.

## Відношення до CML

```text
             shared machine facts
                    │
         ┌──────────┴──────────┐
         ▼                     ▼
 direct Lisp lowering         CML
 proof/bootstrap path      optimizer
         │                     │
         └──────────┬──────────┘
                    ▼
                 target CPU
```

Direct path потрібен не для конкуренції з CML, а як фізичний доказ, що машинний доступ не є семантичною власністю Rust або compiler backend.

## Наукова дисципліна тверджень

Дозволені рівні твердження мають відповідати доказам:

1. **ISA represented in Lisp** — після catalogue proof.
2. **Lisp-owned machine encoding** — після golden-byte tests.
3. **Lisp emits and executes native code** — після фізичного execution witness.
4. **Lisp semantics directly reaches CPU through Lisp-owned lowering** — після semantic witness.
5. **Vertical Lisp architecture** — лише після кількох core semantics, включно хоча б з однією structural Lisp operation, що проходять interpreter/native parity.

Поточний доказ дійшов до рівня 4 для вузького bounded arithmetic witness. Повна vertical Lisp architecture ще не заявляється.

## Наступний сильніший експеримент

Не розширювати каталог заради кількості. Наступна черга доказів:

```text
interpreter/native parity corpus для 0104
        ↓
car / cdr machine data-model witness
        ↓
cons після явного allocation/representation contract
```

Після цього можна обґрунтовано говорити не лише про арифметичний native fast path, а про фізичне представлення самого Lisp data model.

Назва явища не може бути сильнішою за найсильніший експеримент, який його підтримує.
