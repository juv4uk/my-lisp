# Реакція Viveka на шість зовнішніх оглядів `my-lisp`

**Автор:** `wsl-viveka-1` (Viveka)

**Дата:** 2026-08-26

**Метод:** усі шість оглядів прочитані повністю; consequential claims вибірково
перевірені проти поточного дерева. Це незалежна реакція: нотатки інших агентів
не використовувалися як джерело.

## Межа цього документа

Огляди Manus AI і ChatGPT зафіксували репозиторій на `a662dc7`. Поточний стан
уже новіший. Тому твердження нижче поділено на:

- **CONFIRMED** — безпосередньо підтверджено поточними файлами або виконаними
  тестами;
- **HISTORICAL** — коректно для зафіксованого review snapshot, але вже не для
  поточного HEAD;
- **PARTIAL** — має реальне ядро, але огляд підняв силу висновку вище доказу;
- **DISPUTED** — поточний код дає суттєвий контрдоказ;
- **UNKNOWN** — потрібен окремий експеримент.

Самі review-файли не треба переписувати після кожного коміту: це історичні
свідчення з названим snapshot. Актуальний стан має уточнювати окрема реакція або
новий аудит.

## Що всі огляди побачили правильно

### 1. Це вже не навчальний інтерпретатор — **CONFIRMED**

Вісім workspace crates, capability-free core, CLI/host/WASM/LSP/literate
адаптери, значний `lib/*.my` шар, reasoning, Worlds, JTMS, evidence і swarm —
це реальна структура репозиторію. Найточніше коротке визначення з оглядів:
`my-lisp` є малою symbolic-computing system, де одна структурна форма несе
програми, факти, правила, докази й контракти.

### 2. Межа core / host / library — головна архітектурна сила — **CONFIRMED**

`crates/my-lisp/Cargo.toml` досі має порожній `[dependencies]`.
Filesystem/process/TCP живуть у `my-lisp-host`; `lib/*.my` містить те, що мова
може виростити над kernel. WASM використовує canonical `Session`, але не
встановлює native host capabilities. Це не косметичний поділ директорій, а
реальна межа виконання.

### 3. Exactness є семантичною властивістю — **CONFIRMED**

Точні числа, named failures і observable `ErrorKind` входять до ратифікованого
`language-contract.my` 3.0. Сильна сторона тут не просто власний BigInt, а
відмова мовчки міняти exact value на approximation.

### 4. Мова справді росте суттєву систему в `.my` — **CONFIRMED**

`meta-eval.my`, `unify.my`, `reason.my`, `forward.my`, `knowledge.my`,
`world.my`, `content-store.my` і CLIPS importer є виконуваним кодом із тестами,
а не назвами майбутніх модулів. Це найсильніший доказ гасла “grows itself”.

### 5. Evidence discipline реальна — **CONFIRMED, але нерівномірна**

Conformance fixtures, CI, evidence records і named epistemic states вже
змушують claims зустрічатися з executable counterexamples. Водночас не кожне
число або статус у prose автоматично оновлюється; самі огляди показали, як
швидко snapshot claim стає історичним.

## Найважливіші корекції

### Contract 3.0 і draft constitution — не суперечність

- `language-contract.my`: **RATIFIED**, version 3.0.
- `my-lisp-constitution.my`: **draft — not yet ratified**.

Grok і Sarvam коректно описують ратифікований language contract. Manus і
ChatGPT коректно попереджають, що constitution ще draft. Це два різні
артефакти з різними authority roles. Неправильно було б підвищити constitution
до authority лише тому, що вона machine-readable і generated.

### Red CI у Manus — **HISTORICAL**, але його семантичний finding досі живий

На `a662dc7` workspace був red через відсутню metadata для `*argv*`. Коміт
`05dd9b9` додав signature metadata; поточний full-workspace evidence, записаний
у `tasks.my`, повідомляє близько 423 тестів і 0 failures. Отже “CI зараз red”
більше не актуальне.

Але green CI закрив лише discoverability assertion, не довів єдину семантику:

- CLI перед виконанням файла визначає `*argv*` як **list of strings**;
- root environment також реєструє callable `(*argv*)`, який після arity-0
  перевірки повертає **empty vector**;
- `docs/language-core.md` досі називає `*argv*` CLI-only binding.

Тому глибший Manus finding залишається **CONFIRMED**: одна назва має два
несумісні meanings. Metadata-fix зробив gate green, але не вирішив contract.
Потрібне окреме authority decision, а не ще один тест, що випадково освятить
одну з двох поведінок.

### Подвійна реєстрація `string-slice` — **CONFIRMED**

`builtins.rs` реєструє `string-slice` двічі. Перша inline реалізація приймає
лише compact `Value::Number`; друга делегує canonical implementation у
`special_forms::evaluate_string_slice` і мовчки перекриває першу. Користувач
бачить пізню реалізацію, тому це не два observable builtins, але це реальний
maintenance trap і друге джерело семантики. Його слід прибрати атомарно.

### “Повний reasoning pipeline” не дорівнює доведеному масштабу — **PARTIAL**

Pipeline `understand → advise → reason/forward → narrate` існує і тестований.
Це доводить зв'язність та bounded behavior. Це не доводить придатність для
тисяч або мільйонів фактів. Sarvam-критик правильно вимагає профілювання на
100/500/1000 фактах перед словом “scale”. Але його конкретне `O(n²)` я теж не
приймаю як факт без вимірювання й точного аналізу кожного rule path.

### FPGA є незалежним falsification substrate, але не повним доказом G7 — **PARTIAL**

Сам факт іншого фізичного substrate сильніший за другу обгортку того самого
Rust evaluator. Проте повна Rust↔FPGA semantic parity не випливає з існування
репозиторію або частини fixtures. Потрібна machine-readable матриця:
fixture → supported/unsupported → RTL-SIM evidence → HW evidence.

### Великий файл — signal, не failure proof

`crates/my-lisp-cli/src/swarm.rs` має 1654 рядки, `swarm-node/src/main.rs` —
2024. Це справжній maintenance signal і ознака двох coordination generations.
Але твердження “один файл падає — весь mesh падає” не доведене самим LOC.
Модульність треба виводити з change boundaries, tests і failure isolation, а
не лише з довжини файла.

## Де я не погоджуюся з Критиком Sarvam

### Yantra не є “отруєною стрілою” автоматично

`yantra.my` не проникає в language core. Він явно використовує opt-in
`process-run` для `curl` і `bash`, зберігає exit code/stderr, переводить
transport failure у `blocked` і забороняє textual execution claim без
реального tool-result message. Це сильніша evidence boundary, ніж описано в
критиці.

Ризик усе одно реальний: `bash -c`, network endpoint і external model є
високопотужними host capabilities. Тому Yantra має лишатися experimental host
program, а LLM output — hypothesis/proposal, не proof. Але залежність Yantra
від API не робить залежним від API canonical Advice Taker; це різні шари.

### Тримовність не можна скасувати без evidence

Теза “німецьку, мабуть, ніхто не читає” — припущення. Вона не дає права
прибирати DE або переводити всю документацію на одну мову. Реальна проблема —
drift між версіями; правильний перший засіб — generated tables, parity checks і
явна language policy. Рішення про скорочення мов належить власнику й потребує
даних про користування, а не лише вартості підтримки.

### Документація не просто шум

Частина docs є контрактами, evidence, ADR та історією negative results. Їхня
кількість сама по собі не є defect. Defect виникає, коли читач не може
відрізнити current authority від historical/proposal. Потрібні lifecycle
metadata та navigation, а не масове видалення.

## Findings, які варто перетворити на роботу

### P0 — semantic consistency

1. **Вирішити `*argv*` authority:** CLI-injected value чи callable core
   builtin. Поточний hybrid суперечить власній документації.
2. **Залишити одну registration `string-slice`:** canonical implementation +
   наявні Unicode/clamping regressions.

### P1 — security і documentation truth

3. **Scoped host capabilities:** filesystem read/write roots і TCP
   hosts/ports перед використанням native CLI як autonomous-agent runtime.
   Поточні `std::fs::*`, unrestricted connect і `0.0.0.0` listen підтверджують
   concern Manus/ChatGPT.
4. **Documentation lifecycle:** прибрати або генерувати stale run dates;
   виправити українську згадку майбутнього C core і duplicated README crate
   entries; не редагувати історичні review snapshots.

### P2 — experiments before expansion

5. **Reasoning scale benchmark:** 100/500/1000 facts, separate backward,
   forward and JTMS cases; record time, allocations and result parity.
6. **Cross-substrate conformance matrix:** fixture-level evidence, окремі
   стани native-pass / RTL-sim-pass / synth-pass / hardware-pass.
7. **Swarm module boundary audit:** знайти change clusters і failure domains
   перед refactor; LOC — лише trigger для аудиту.

## Мій підсумок

Найсильніша частина `my-lisp` — не окрема feature і навіть не Lisp syntax.
Це збережена межа між precise symbolic core та зовнішніми, слабшими або
потужнішими шарами: host capabilities, LLM, Sanskrit research, swarm, FPGA.
Коли ця межа явна, широта екосистеми є силою. Коли одна назва отримує дві
семантики (`*argv*`) або proposal звучить як implementation, широта стає
джерелом самообману.

Тому моя реакція ближча до спільного висновку оглядів, але жорсткіша в одному:
поточний green signal не повинен закривати semantic contradiction. Спочатку
одна семантика на одну назву, одна registration на primitive і точний
evidence class; після цього — performance, scale та нові substrates.
