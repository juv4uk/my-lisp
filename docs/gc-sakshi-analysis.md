# GC: аналіз Сакші — що я додаю після двох зовнішніх оглядів

**Статус:** аналіз + позиція · **Дата:** 2026-08-23
**Автор:** Сакші (sākṣī, ox-alpha)
**Читається разом з:** gc-m0-design.md (ChatGPT), gc-holistic-map.md (Manus AI)

Обидва зовнішні документи якісні й зійшлися на висновку (precise STW
non-moving mark-sweep, tests first). Я погоджуюсь із висновком. Але обидва
пропускають чотири речі, які для **нашої** екосистеми важливіші за вибір
алгоритму. Це мій внесок.

## 1. Головний ризик — не collector, а міграційний шлях

Обидва документи детально розбирають алгоритми і майже не торкаються того,
що насправді боляче: сьогодні `Value` володіє даними через `Rc<str>`,
`Rc<[Expr]>`, `Rc<dyn Fn>` по всьому eval/. Перехід на heap+ObjectId
означає переписати **кожну сигнатуру evaluator'а**.

```text
Вибір collector'а — тиждень роботи.
Міграція сигнатур Value → Ref(ObjectId) — місяці і головне джерело регресій.
Тому проєктувати треба МІГРАЦІЮ, а не collector.
```

Конкретно: першим кроком має бути не ManagedHeap, а **модуль-фасад**
`value_storage.rs`, за яким ховається сьогоднішній Rc. Evaluator працює
через фасад; під ним потім без зміни викликів зʼявляється heap. Той самий
патерн, що physical core/host split (f565f66): спершу шов, потім пересадка.

## 2. Екосистемний патерн: incarnation handles вже винайдені

У swarm-node минулого тижня я виправляв ABA-баг антиентропії: подія
ідентифікувалась парою `(node, seq)`, і реінкарнація ноди під тим самим id
створювала колізії. Фікс: `(node, incarnation, seq)`.

GC M0 пропонує рівно той самий патерн: `ObjectId { slot, generation }`.
Це не збіг — це **одна системна відповідь на одну системну проблему**:
сталий handle у просторі, де обʼєкти вмирають і slot'и перевикористовуються.

Висновок для екосистеми: варто зафіксувати цей патерн як **спільний контракт**
(наприклад, сторінка в docs/ або запис у memory-layout-contract.my):
«identity = (namespace, incarnation, ordinal)». Тоді swarm-node, GC,
і майбутній FPGA heap використовуватимуть одну модель — і баг одного стане
уроком для всіх.

## 3. Контракт перед кодом — у стилі екосистеми

Manus згадує FPGA одним рядком; я б зробив це центральним. У нас вже є
прецедент: `memory-layout-contract.my` зарезервував NaN-boxing слот під
TAG_PRIMITIVE «from day one» — і contract 2.1 це використав.

Пропозиція: **gc-object-contract.my** ДО реалізації Rust:

- object tags та layout child-references;
- root categories (глобальний env / lexical chain / stack / host handles);
- named outcomes: OOM, timeout, stress-mode;
- identity contract з п.2.

Тоді fpga-lisp і CML можуть реалізувати той самий контракт незалежно,
а Rust-collector стає однією з реалізацій, не джерелом істини.

## 4. Instrumentation FIRST — мій спір із порядком ChatGPT

ChatGPT каже: «спершу failing tests, потім ManagedHeap for Pair».
Я каже: **спершу профіль allocation**, бо без нього ми проєктуємо всліпу.

У нас уже є seam: `cons_limit` в Environment (environment.rs). Розширити його
до лічильника + періодичної статистики — маленька зміна, нуль ризику.
Далі прогнати WSM-24 (2220 яєць), yantra agent, conformance suite і
відповісти на питання:

- скільки живих обʼєктів у піку? (визначає розмір mark worklist)
- який відсоток помирає молодим? (виправдовує чи ні generational)
- де взагалі тиск на памʼять? (можливо, його немає і GC не горить)

Якщо WSM-24 на 2220 яйцях вміщується в cons_limit без колапсу — M0 можна
чесно відкласти. Якщо ні — маємо конкретні числа для дизайну. Обидва
результати корисні; проєктування без них — ні.

## 5. GC як свідок: звʼязка з epistemic.my

`(gc-stats)` з ChatGPT-дизайну — це не просто debug API. У нашій мові
runtime-інтроспекція може бути **observable evidence** у сенсі epistemic.my:

```lisp
(observation (kind 'gc-stats) (payload (gc-stats)))
```

Collector стає ще одним witness, чиї твердження мають provenance
(модель, timestamp, ліміти). Це узгоджує memory management із
executable epistemology замість того, щоб тримати їх окремими світами.

## 6. Мій вердикт і порядок

| Крок | Що | Хто/коли |
|---|---|---|
| 0a | value_storage.rs фасад над Rc (нуль поведінкових змін) | перший PR |
| 0b | allocation counters поверх cons_limit + прогони WSM-24/yantra | там саме |
| рішення | якщо тиск є → gc-object-contract.my → M0 за ChatGPT-планом | після чисел |
| паралельно | identity contract (incarnation pattern) у docs | будь-коли |

Позиція: **не починати M0 зараз**. Починати instrumentation. Це розходження
з ChatGPT-порядком («failing tests спершу») принципове: failing тест на
missing root має сенс лише коли є real allocation pressure, який його
провокує. Інакше ми тестуємо вигаданий workload.

## 7. Головна думка

Найкращий GC для my-lisp — той, який зросте з виміряного тиску, буде описаний
контрактом до коду, і поділить модель ідентичності з рештою екосистеми.
Алгоритм — найлегша частина.

