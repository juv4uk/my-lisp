# my-lisp language core axioms · Аксіоми ядра my-lisp · my-lisp-Sprachkern-Axiome

**Status: draft, not yet ratified.** This document precedes `my-lisp-constitution.json` — the JSON becomes the executable proof of these axioms, not the starting point. Nothing here is final until discussed and agreed on explicitly.

**Статус: чернетка, ще не затверджена.** Цей документ передує `my-lisp-constitution.json` — JSON стане виконуваним доказом цих аксіом, не відправною точкою. Нічого тут не остаточне, поки не обговорено й не узгоджено явно.

**Status: Entwurf, noch nicht ratifiziert.** Dieses Dokument geht `my-lisp-constitution.json` voraus — das JSON wird der ausführbare Beweis dieser Axiome, nicht der Ausgangspunkt. Nichts hier ist endgültig, bis es ausdrücklich besprochen und vereinbart wurde.

## Project principles · Принципи проєкту

Above the axioms — a separate, closed list of five, agreed on 2026-08-09, that filters how the axioms below get written and read. Not axioms themselves (an axiom is a claim about the language; these are claims about how *we* talk about the language). Deliberately stopped at five rather than left open-ended — a sixth or seventh would need to earn its place the same way these did, not extend the list by default.

Над аксіомами — окремий, закритий список із п'яти пунктів, узгоджений 2026-08-09, що фільтрує, як пишуться й читаються аксіоми нижче. Це не самі аксіоми (аксіома — твердження про мову; це твердження про те, як *ми* говоримо про мову). Свідомо зупинено на п'яти, не залишено відкритим — шостий чи сьомий пункт мав би заслужити своє місце так само, як ці, а не продовжувати список за замовчуванням.

**1. Write about possibilities, not limitations.** The same instinct that makes Lisp itself feel generative rather than restrictive. This is why the axioms below split into generative (G) and safety (S) axioms instead of one flat list of "must"s — see the note at the top of that section for where a prohibition is still the honest choice.

**1. Писати про можливості, не про обмеження.** Той самий інстинкт, що робить сам Lisp генеративним, не забороняючим. Саме тому аксіоми нижче розділені на генеративні (G) і безпекові (S), а не один плаский список "маєш" — див. примітку на початку того розділу, де заборона все ще чесний вибір.

**2. Be Lisp in the full sense of the word.** Not "Lisp-flavored syntax" — genuinely of the Lisp family, in the way that survived every historical fork (MacLisp/InterLisp/Scheme/Common Lisp/Clojure), not the way any single dialect happened to do things. From the earlier discussion of what actually unites almost all Lisps: **homoiconicity** (code is data — G3, G7) and **a minimal, closed core that grows the rest of the language from inside itself** (G2, G4, G5) are the two properties that kept reappearing across 65 years of divergence; things like dynamic scoping (an implementation accident, already rejected — lexical scoping is already the choice made in code) did not survive and are not part of "being Lisp." Being Lisp in the full sense means holding onto the properties that proved durable, not imitating the surface syntax of any one historical dialect.

**2. Бути Lisp-ом у повному розумінні цього слова.** Не "синтаксис у стилі Lisp" — справді з родини Lisp, у тому сенсі, що пережив кожен історичний розкол (MacLisp/InterLisp/Scheme/Common Lisp/Clojure), не в тому сенсі, як конкретний діалект випадково щось робив. З раніших роздумів про те, що насправді об'єднує майже всі Lisp-и: **гомоіконність** (код — це дані — G3, G7) і **мінімальне, замкнене ядро, що вирощує решту мови зсередини себе** (G2, G4, G5) — це дві властивості, що знову й знову з'являлись за 65 років розбіжностей; такі речі, як динамічне зв'язування (випадковість реалізації, уже відкинута — лексичне зв'язування вже є вибором у коді), не пережили й не є частиною "бути Lisp-ом". Бути Lisp-ом у повному розумінні означає тримати саме ті властивості, що довели свою тривкість, не наслідувати поверхневий синтаксис якогось одного історичного діалекту.

**3. Build the reasoning machine — McCarthy's own documented goal, extended by the author's own.** Two related but distinct claims, kept separate rather than blurred into one unnamed "he dreamed / I dream": (a) McCarthy's *documented* 1958 goal was the **Advice Taker** — a system you could tell facts to in a formal notation, that would draw logically valid conclusions from them, the way a person acting on advice does; this is not speculation about his views, it's in the paper itself (`docs/mccarthy-vision.md`). Lisp was the notation he built *to reach* that goal, not the goal itself. (b) The author's own extension, stated in `private/lisp-to-knowledge.md`: a modern hybrid where a neural layer handles the fuzzy human interface (natural language, ambiguity) and a precise symbolic core (exactly what `lib/reason.my`/`lib/forward.my`/`lib/unify.my`/`lib/knowledge.my` already are) does the actual reasoning, with explicit provenance — "X, because A and B, and rule C" instead of "the answer is approximately X." This principle exists so Tier 3 (ECOSYSTEM CONFORMANCE — the symbolic-reasoning layer) is never treated as an optional add-on to "a Lisp" — it's the actual point, for both McCarthy's own stated reason for building Lisp and the author's own reason for continuing it.

**3. Реалізувати розумну машину — задокументована ціль МакКарті, продовжена власною ціллю автора.** Два пов'язані, але окремі твердження, не змішані в одне безіменне "він мріяв / я мрію": (a) *задокументована* ціль МакКарті 1958 року — **Advice Taker**: система, якій можна повідомляти факти формальною нотацією, і яка робитиме логічно правильні висновки з них, так само як людина, що діє за порадою; це не здогад про його погляди, це в самій статті (`docs/mccarthy-vision.md`). Lisp був нотацією, яку він побудував, *щоб дійти* до цієї цілі, не самою ціллю. (б) Власне продовження автора, сформульоване в `private/lisp-to-knowledge.md`: сучасний гібрид, де нейронний шар обробляє нечіткий людський інтерфейс (природну мову, неоднозначність), а точне символьне ядро (саме те, чим уже є `lib/reason.my`/`lib/forward.my`/`lib/unify.my`/`lib/knowledge.my`) виконує саме міркування, з явним походженням — "X, тому що A і B, а з правила C" замість "відповідь приблизно X". Цей принцип існує, щоб Рівень 3 (ЕКОСИСТЕМНА КОНФОРМНІСТЬ — шар символьного міркування) ніколи не трактувався як опційна прибудова до "якогось Lisp-а" — це і є сама суть, і для задокументованої причини МакКарті будувати Lisp, і для власної причини автора продовжувати цю справу.

**4. Cross-platform-ness, or more simply: universality.** Not a testing convenience — the falsifiability test for G6/G7. A language that only ever lived in one implementation hasn't actually *proven* "conformance is defined by observable behavior, not internal architecture" — it's only asserted it. `my-lisp` already commits to three real, physically different substrates, not hypothetically: this Rust core (software), a future C core (embedded — real bignum-capable exact arithmetic is a genuinely hard open problem there, not assumed away), and `fpga-lisp` (hardware — gates, not a soft-core CPU running a C interpreter). Today's own session is direct evidence this isn't aspirational: `fpga-lisp`'s simulation was run live and `(car (cons 'a 'b))` was confirmed correct on the actual hardware model, not just claimed. Universality means the same axioms hold whether the machine is a Rust process, a C binary on a microcontroller, or logic gates on silicon — not that every substrate must be equally fast, equally unbounded, or equally convenient (that's what the safety axioms' resource-limit clause already covers). Honest status as of 2026-08-09: `fpga-lisp` has hardware-verified milestones M01–M05 (tagged words, heap, `cons`, `car`/`cdr`, `atom`/`eq`) — five of McCarthy's seven primitives, confirmed by live simulation, not asserted. It does not yet run `conformance.json` itself; that is its own roadmap item 28 of 30 (`fpga-lisp/docs/lisp-machine-plan.md`). So today's proof is at the primitive level — the same operations really do work on two physically different substrates — not yet at the full-conformance level (G7's "same expression, same meaning everywhere" tested against the identical fixture set on both). The claim should be read at the strength it currently has, not the strength it will have once step 28 lands.

**4. Кросплатформеність, або простіше — універсальність.** Не зручність тестування — тест на фальсифіковність для G6/G7. Мова, що жила лише в одній реалізації, насправді не *довела* "конформність визначається спостережуваною поведінкою, не внутрішньою архітектурою" — лише заявила це. `my-lisp` уже зобов'язується перед трьома реально різними, не гіпотетичними, фізичними субстратами: це Rust-ядро (софт), майбутнє C-ядро (embedded — реальна bignum-здатна точна арифметика там — справді складна відкрита задача, не вигадана заздалегідь відповідь), і `fpga-lisp` (залізо — вентилі, не soft-core CPU, що виконує C-інтерпретатор). Сама сьогоднішня сесія — пряме свідчення, що це не мрія: симуляцію `fpga-lisp` запущено наживо, і `(car (cons 'a 'b))` підтверджено коректним на реальній апаратній моделі, не лише заявлено. Універсальність означає, що ті самі аксіоми діють незалежно від того, чи машина — процес Rust, чи C-бінарник на мікроконтролері, чи логічні вентилі на кремнії — не те, що кожен субстрат має бути однаково швидким, однаково необмеженим чи однаково зручним (це вже покриває пункт про обмеження ресурсів у безпекових аксіомах). Чесний статус на 2026-08-09: `fpga-lisp` має апаратно перевірені мілстоуни M01–M05 (теговані слова, heap, `cons`, `car`/`cdr`, `atom`/`eq`) — п'ять із семи примітивів МакКарті, підтверджено живою симуляцією, не заявлено на слово. Він ще не запускає сам `conformance.json` — це власний пункт 28 з 30 у дорожній карті (`fpga-lisp/docs/lisp-machine-plan.md`). Тобто сьогоднішній доказ — на рівні примітивів (ті самі операції справді працюють на двох фізично різних субстратах), а не ще на рівні повної конформності (G7: "той самий вираз, те саме значення всюди", перевірене тим самим набором фікстур на обох). Твердження варто читати з тією силою, яку воно має зараз, а не з тією, яку матиме після кроку 28.

**5. Maximum awareness of today's technology, applied to symbolic AI.** Distinct from principle 3: principle 3 names the destination (a reasoning machine); this one is about staying current *on the way there*, instead of treating classical symbolic AI as a museum piece frozen in 1958 or 1985 (CLIPS's own era). Concretely, not hypothetically: `lib/clips-import.my`'s whole method — testing the reasoning engine against real historical rule bases from CLIPS's actual distribution, not hand-picked toy examples — only exists because the project used modern tooling (a real Rust toolchain, GitHub CI, `iverilog` for hardware simulation) to hold a 1985 idea to a standard it never had to meet on its own hardware. `private/lisp-to-knowledge.md` already states the specific application of this principle that matters most: a modern LLM is not a competitor to the symbolic core, it's the fuzzy natural-language interface *in front of* it — precision stays with the symbolic layer, ambiguity-handling goes to the neural layer, neither pretends to be the other. This session is itself an instance of the principle, not just a description of it — an LLM-based coding assistant was the tool used to build and verify all of today's symbolic-AI work.

**5. Максимальна обізнаність у сьогоднішніх технологіях, застосована до символьного ШІ.** Окремо від принципу 3: принцип 3 називає ціль (машина, що міркує); цей — про те, щоб лишатись сучасним *на шляху туди*, а не трактувати класичний символьний AI як музейний експонат, застиглий у 1958 чи 1985 (власна епоха CLIPS). Конкретно, не гіпотетично: увесь метод `lib/clips-import.my` — перевірка рушія міркування на реальних історичних базах правил зі справжнього дистрибутиву CLIPS, не на дібраних вручну іграшкових прикладах — існує лише тому, що проєкт використав сучасні інструменти (справжній Rust-тулчейн, GitHub CI, `iverilog` для апаратної симуляції), щоб приміряти ідею 1985 року до стандарту, якого вона ніколи не мала на власному залізі. `private/lisp-to-knowledge.md` уже формулює конкретне застосування цього принципу, яке важить найбільше: сучасна LLM — не конкурент символьному ядру, а нечіткий інтерфейс природної мови *перед* ним — точність лишається за символьним шаром, обробка неоднозначності йде в нейронний, і жоден не вдає, що він — інший. Ця сама сесія — не просто опис принципу, а його приклад у дії: LLM-асистент для програмування був інструментом, яким сьогодні побудовано й перевірено всю роботу над символьним AI.

## Why a separate document from `conformance.json`

`conformance.json` currently mixes three different ranks of claim in one flat list — a core-language fact (`(car '(a b))`), a stdlib fact (`(map ...)`), and a full symbolic-reasoning proof tree (`reason`/`unify`) all sit at the same level. That ambiguity has a real cost: if a future `fpga-lisp` core fails a `reason-explain` fixture, there is no way to tell from the file alone whether that means "this isn't my-lisp" or merely "this implementation hasn't loaded `lib/reason.my` yet." These axioms exist to make that distinction explicit *before* any fixture gets sorted into a tier.

## Чому окремий документ від `conformance.json`

Зараз `conformance.json` змішує три різні ранги тверджень в одному плаский списку — факт про ядро мови (`(car '(a b))`), факт про stdlib (`(map ...)`) і повне дерево доведення символьного міркування (`reason`/`unify`) стоять на одному рівні. Ця двозначність має реальну ціну: якщо майбутнє ядро `fpga-lisp` провалить фікстуру `reason-explain`, з самого файлу неможливо зрозуміти, чи це означає "це не my-lisp", чи просто "ця реалізація ще не завантажила `lib/reason.my`". Ці аксіоми існують, щоб зробити цю різницю явною *до того*, як будь-яка фікстура потрапить у якийсь рівень.

## The three tiers · Три рівні

```
MY-LISP LANGUAGE CONTRACT
│
├── 1. CORE SEMANTICS       — every conforming implementation must have this
│   quote atom eq car cdr cons cond, lambda, evaluation rules,
│   truth/NIL, symbols, pairs
│
├── 2. LANGUAGE CONTRACT    — every conforming implementation must have this
│   arithmetic (exact/inexact), def, defmacro, errors, read/eval
│
└── 3. ECOSYSTEM CONFORMANCE — an implementation can be "my-lisp" without
    this loaded yet; this tests a *library*, not the language itself
    core.my, unify.my, reason.my, knowledge.my, literate markdown, CLIPS
```

**Українською:** Рівень 1 (СЕМАНТИКА ЯДРА) — обов'язковий для кожної конформної реалізації: `quote atom eq car cdr cons cond`, `lambda`, правила обчислення, істина/NIL, символи, пари. Рівень 2 (КОНТРАКТ МОВИ) — теж обов'язковий: арифметика (exact/inexact), `def`, `defmacro`, помилки, `read`/`eval`. Рівень 3 (ЕКОСИСТЕМНА КОНФОРМНІСТЬ) — реалізація може бути "my-lisp" без цього завантаженого; це перевіряє *бібліотеку*, не саму мову: `core.my`, `unify.my`, `reason.my`, `knowledge.my`, literate markdown, CLIPS.

## Two kinds of axiom · Два види аксіом

Rewritten after an explicit decision (2026-08-09): **our philosophy is to write about possibilities, not limitations** — the same instinct that makes Lisp itself feel generative rather than restrictive. But not every axiom can honestly be phrased that way. P5–P7 exist *because* something silently broke without them — the dotted-pair fix earlier the same day is a literal example of a silent break these axioms are meant to prevent. A prohibition addresses a real, specific risk of quiet failure; a possibility does not, by itself. So the axioms below split into two kinds, not one flat list: **generative axioms** (G) say what my-lisp *makes possible*; **safety axioms** (S) say what no conforming implementation may *silently do* — the same shape the Bill of Rights takes ("Congress shall make no law...") for exactly the same reason: a stated possibility doesn't stop a specific abuse, but an explicit prohibition does.

Переписано після явного рішення (2026-08-09): **наша філософія — писати про можливості, не про обмеження** — той самий інстинкт, що робить сам Lisp генеративним, не забороняючим. Але не кожну аксіому чесно можна сформулювати так. P5–P7 існують *тому*, що без них щось мовчки ламалось — dotted-pair-фікс того самого дня буквальний приклад мовчазної поломки, якій ці аксіоми мають запобігати. Заборона адресує конкретний ризик тихого провалу; можливість сама собою — ні. Тому аксіоми нижче розділені на два види, не один плаский список: **генеративні аксіоми** (G) кажуть, що my-lisp *робить можливим*; **безпекові аксіоми** (S) кажуть, чого жодна конформна реалізація не має права робити *мовчки* — та сама форма, яку має Білль про права ("Congress shall make no law...") з тієї самої причини: заявлена можливість не зупиняє конкретне зловживання, а явна заборона — так.

## Generative axioms — what my-lisp makes possible · Генеративні аксіоми — що my-lisp робить можливим

### G1 — A value's meaning can be fully defined by observable behavior · Значення value може бути повністю визначене спостережуваною поведінкою

Freeing every implementation to choose its own representation. `CONS = a BRAM address`, `symbol = a 28-bit id`, `integer = i64`, `environment = a hash table` are all implementation facts, not language facts — the contract only ever says things like `(car (cons 'a 'b)) => a`, never how `cons` is stored.

Це звільняє кожну реалізацію обирати власне представлення. `CONS = адреса в BRAM`, `символ = 28-бітний id`, `ціле = i64`, `середовище = хеш-таблиця` — усе це факти реалізації, не факти мови; контракт завжди каже лише щось на кшталт `(car (cons 'a 'b)) => a`, ніколи — як саме `cons` зберігається.

**Свідчення сьогодні:** дискусія про `fpga-lisp` — 28-бітний регістр там vs довільна точність у Rust — саме та різниця, яку ця аксіома робить неважливою для контракту.

### G2 — Every value can be built from just two things: atoms and pairs · Кожне значення можна побудувати лише з двох речей: атомів і пар

The whole data universe grows from one structural rule: `(a b c)` is sugar for `(a . (b . (c . ())))`. `cons`/`car`/`cdr` are the algebra that builds and takes apart this one structure, not a grab-bag of "list utilities."

Увесь всесвіт даних росте з одного структурного правила: `(a b c)` — цукор для `(a . (b . (c . ())))`. `cons`/`car`/`cdr` — це алгебра, що будує й розбирає саме цю одну структуру, не набір розрізнених "утиліт для списків".

**Перевірено сьогодні, не гіпотетично:** до 2026-08-09 ця можливість була порушена в самій реалізації — `'(p . 0)` читався reader'ом як звичайний трьохелементний список, не як `cons('p, 0)`, попри те що обидва друкувалися однаково. `read ∘ print ≠ identity` для частини простору значень. Виправлено тим самим днем (`ExprKind::Pair`, `crates/my-lisp/src/parser.rs`) — саме тому цю аксіому можна тепер писати з чистою совістю.

### G3 — Program structure can be inspected, transformed, and built like any other value · Структуру програми можна оглядати, трансформувати й будувати, як і будь-яке інше значення

Not just "support `'(+ 1 2)`" — the deeper claim: the syntactic structure of a program is a value of the language. `quote`, `read`, `eval`, macros, and a metacircular evaluator (`lib/meta-eval.my`) are all consequences of this, not separate features bolted on.

Не просто "підтримати `'(+ 1 2)`" — глибше твердження: синтаксична структура програми є значенням мови. `quote`, `read`, `eval`, макроси й метациркулярний евалюатор (`lib/meta-eval.my`) — усе це наслідки цього факту, не окремі прибудовані можливості.

### G4 — A minimal core can grow an entire language inside itself · Мінімальне ядро може вирощувати всю мову всередині себе

Before reaching for a new primitive, ask what the existing core can already express — this is a filter applied to every future proposal, not a one-time design choice.

Перш ніж тягнутись за новим примітивом, спитати, що наявне ядро вже може виразити — це фільтр, застосовний до кожної майбутньої пропозиції, не одноразове дизайн-рішення.

**Приклад із сьогоднішньої роботи:** `exists`/`forall` (Крок 15) додано в `lib/forward.my`, не в Rust — обидва виражаються через уже наявний `match-conditions`, без нового примітиву.

### G5 — Anything expressible within the language can live above the implementation boundary · Усе, що виразне мовою, може жити над межею реалізації

`map`/`filter`/`reduce` live in `lib/core.my`, not the Rust core, precisely because they're already expressible using only the core — a library, not a fixed part of the machine. The mirror image of G4, applied to what's *already there*: G4 gates new additions, G5 is the ongoing test.

`map`/`filter`/`reduce` живуть у `lib/core.my`, не в Rust-ядрі, саме тому, що вже виразні через саме ядро — бібліотека, не фіксована частина машини. Дзеркальне відображення G4, застосоване до того, що *вже є*: G4 фільтрує нові додавання, G5 — постійна перевірка.

### G6 — Conformance can be defined purely by observable behavior · Конформність можна визначити суто спостережуваною поведінкою

Letting radically different machines — Rust, a future C core, `fpga-lisp` — all genuinely be the same language. A future HDL core needs no `Rc`, no heap shape matching Rust's, no shared evaluator code — only the same answers. This is `conformance.json`'s entire reason for existing.

Це дозволяє геть різним машинам — Rust, майбутньому C-ядру, `fpga-lisp` — справді бути однією мовою. Майбутньому HDL-ядру не потрібен `Rc`, форма купи, що збігається з Rust-реалізацією, чи спільний код evaluator'а — лише ті самі відповіді. Це вся причина існування `conformance.json`.

### G7 — The same expression can mean the same thing everywhere · Той самий вираз може означати те саме всюди

The unifying possibility all the others serve. Rust, a future C core, `fpga-lisp` — all implementations of one abstract system, not separate dialects that happen to share a name.

Об'єднувальна можливість, якій служать усі інші. Rust, майбутнє C-ядро, `fpga-lisp` — усе це реалізації однієї абстрактної системи, не окремі діалекти, що випадково поділяють назву.

### G8 — The absence of any element and the absence of truth can be the same value · Відсутність будь-якого елемента й відсутність істини можуть бути тим самим значенням

Found as a real gap while walking `conformance.json` fixture-by-fixture (`docs/conformance-tier-map.md`, fixtures #10/#20) — `cond`/NIL semantics were already implied by the Tier 1 definition but never stated as an axiom. Resolved by doing what McCarthy himself did in Lisp 1.5, not what Scheme later did: `'()` is both the empty list and the canonical false — one value serves two roles, not two separate "nothing"-shaped values to track (`cond` selects the first clause whose test is not `'()`; `'()` itself is that clause's failure case). This is the same minimal-core instinct as G4, applied to values instead of primitives: fewer distinct concepts of "nothing," not more. Scheme's later split of `'()`/`#f` is a different, equally legitimate design — not a correction of an error — but it is *not* my-lisp's choice; my-lisp's own tests (`(cond (() 'wrong) (t 'right))`) already commit to McCarthy's original conflation, this axiom just says so out loud instead of leaving it implicit.

Знайдено як реальну прогалину під час проходу по `conformance.json` фікстура за фікстурою (`docs/conformance-tier-map.md`, фікстури #10/#20) — семантика `cond`/NIL уже малась на увазі визначенням Рівня 1, але ніколи не була сформульована як аксіома. Вирішено так, як зробив сам МакКарті в Lisp 1.5, не так, як пізніше зробив Scheme: `'()` — водночас порожній список і канонічна хиба — одне значення служить двом ролям, а не два окремі "нічого"-подібні значення, які треба відстежувати (`cond` обирає першу гілку, чий тест не `'()`; сам `'()` — це випадок провалу цієї гілки). Це той самий інстинкт мінімального ядра, що й G4, застосований до значень, не примітивів: менше окремих понять "нічого", не більше. Пізніший розділ `'()`/`#f` у Scheme — інший, так само законний дизайн — не виправлення помилки — але це *не* вибір my-lisp; власні тести my-lisp (`(cond (() 'wrong) (t 'right))`) уже зобов'язуються перед оригінальним суміщенням МакКарті, ця аксіома лише каже це вголос замість того, щоб лишати неявним.

## Safety axioms — what no conforming implementation may silently do · Безпекові аксіоми — чого жодна конформна реалізація не має права робити мовчки

### S1 — Never silently turn an exact value into an approximation · Ніколи мовчки не перетворювати точне значення на наближення

`(/ 1 3)` means exactly `1/3`, not `0.333...`. A concrete machine may have real resource limits and refuse the operation (`NumericOverflow` or similar) — but it must never quietly turn `1/3` into `0.333343` and pretend that's the same value.

`(/ 1 3)` означає точно `1/3`, не `0.333...`. Конкретна машина може мати реальні обмеження ресурсів і відмовити у виконанні операції (`NumericOverflow` чи подібне) — але вона ніколи не має мовчки перетворити `1/3` на `0.333343`, вдаючи, що це те саме значення.

**Відкрите питання, свідомо не вирішене тут:** чи *кожна* майбутня реалізація (включно з `fpga-lisp`, де регістр — 28 біт) мусить підтримувати довільну точність, чи може законно лишитись обмеженою й чесно провалюватись за межею — це вже задокументовано як відкрите в `tests/fixtures/README.md`, не питання цього документа.

### S2 — Never fail silently — every failure is a named, observable outcome · Ніколи не провалюватись мовчки — кожен провал є названим, спостережуваним результатом

`(car 'a)` doesn't just "not work" — it produces a specific, named kind of failure (`Type`). The wording of the message may differ across implementations and languages; the *category* is the contract. `conformance.json` already tests exactly five categories: `Parse`, `UnknownSymbol`, `Arity`, `Type`, `InvalidForm`.

`(car 'a)` не просто "не працює" — вона видає конкретний, названий вид провалу (`Type`). Формулювання повідомлення може відрізнятись між реалізаціями й мовами; контракт — саме *категорія*. `conformance.json` уже перевіряє рівно п'ять категорій: `Parse`, `UnknownSymbol`, `Arity`, `Type`, `InvalidForm`.

### S3 — Never let a resource limit silently redefine an operation's meaning · Ніколи не дозволяти обмеженню ресурсу мовчки переозначити сенс операції

4096 cons cells on an FPGA is a legitimate capability limit — the correct response is `OutOfMemory`, not a `cons` that quietly starts overwriting old cells. Bounded integers are legitimate — the correct response to overflow is a named error, not silent wraparound. No filesystem is a legitimate capability boundary, not a defect. **Bounded implementations are acceptable; incompatible semantics are not.**

4096 cons-комірок на FPGA — законне обмеження можливостей: правильна відповідь — `OutOfMemory`, не `cons`, що мовчки починає перезаписувати старі комірки. Обмежені цілі числа — законні: правильна відповідь на переповнення — названа помилка, не тихе загортання (wraparound). Відсутність файлової системи — законна межа можливостей, не дефект. **Обмежені реалізації прийнятні; несумісна семантика — ні.**

## Deliberately left open — not decided here · Свідомо залишено відкритим — не вирішено тут

- **Arbitrary-precision rationals mandatory for every implementation?** Open (see S1's note, and `tests/fixtures/README.md`).
  **Довільна точність раціональних чисел обов'язкова для кожної реалізації?** Відкрито (див. примітку до S1 і `tests/fixtures/README.md`).
- **Lisp-1 vs Lisp-2 (one namespace vs two for functions/variables).** my-lisp is currently a Lisp-1 in practice (`def` binds one thing per name). Not yet examined carefully enough to state as a settled axiom — and not McCarthy's own decision to begin with; this split emerged after him, in the Scheme/Common Lisp era.
  **Lisp-1 проти Lisp-2 (один простір імен проти двох для функцій/змінних).** my-lisp зараз практично Lisp-1 (`def` зв'язує одну річ на ім'я). Ще не розглянуто достатньо ретельно, щоб сформулювати як усталену аксіому — і це не рішення самого МакКарті: цей розкол виник уже після нього, в епоху Scheme/Common Lisp.
- **Dynamic vs lexical scoping.** Already decided in code (lexical) and not treated as open — early Lisp's dynamic scoping is documented as an implementation accident of the 1960 IBM 704 stack design, not a considered semantic choice, so there is nothing to preserve here.
  **Динамічне проти лексичного зв'язування.** Уже вирішено в коді (лексичне) і не трактується як відкрите — динамічне зв'язування раннього Lisp задокументоване як побічний ефект реалізації стека IBM 704 1960 року, не свідомий семантичний вибір, тож тут нічого зберігати.

## Next step · Наступний крок

**Done, 2026-08-09:** all 66 `tests/fixtures/conformance.json` fixtures walked and tagged (`docs/conformance-tier-map.md`), a gap found and closed (G8), and `my-lisp-constitution.json` built as a self-contained, generated projection — not a hand-maintained second copy. `tests/fixtures/conformance.json` (facts, append-only, unchanged) and `tests/fixtures/conformance-tier-map.json` (tags, index-aligned) are the two real sources of truth; `scripts/build-constitution.py` combines them plus this document's principle/axiom text into `my-lisp-constitution.json`. `crates/my-lisp/tests/mccarthy.rs`'s `constitution_json_stays_in_sync_with_conformance_json` fails loudly if someone edits one file and forgets to regenerate. (Later the same day: the one literate-Markdown fixture was removed as redundant with `crates/my-lisp-literate/tests/literate_offsets.rs`, so the file now holds 65 — see `docs/conformance-tier-map.md`.)

Still open: whether `conformance.json` itself should ever physically split into `language-core.json` / `stdlib.json` / `symbolic.json` by tier, now that the tagging exists in `conformance-tier-map.json` regardless of whether the underlying file's shape changes; and the ratification question itself — this whole document, and `my-lisp-constitution.json`, remain drafts until explicitly ratified, at which point the constitution becomes read-only, matching the release-tag immutability this project already practices elsewhere (`docs/versioning.md`).

**Зроблено, 2026-08-09:** усі 66 фікстур `tests/fixtures/conformance.json` пройдено й позначено (`docs/conformance-tier-map.md`), знайдено й закрито прогалину (G8), і побудовано `my-lisp-constitution.json` як самодостатню, згенеровану проекцію — не другу копію, яку підтримують вручну. `tests/fixtures/conformance.json` (факти, append-only, незмінний) і `tests/fixtures/conformance-tier-map.json` (теги, вирівняні за індексом) — два реальні джерела правди; `scripts/build-constitution.py` комбінує їх разом із текстом принципів/аксіом цього документа в `my-lisp-constitution.json`. Тест `constitution_json_stays_in_sync_with_conformance_json` у `crates/my-lisp/tests/mccarthy.rs` гучно провалюється, якщо хтось відредагує один файл і забуде перегенерувати. (Пізніше того ж дня: literate-Markdown фікстуру видалено як надлишкову відносно `crates/my-lisp-literate/tests/literate_offsets.rs`, тож файл тепер має 65 — див. `docs/conformance-tier-map.md`.)

Досі відкрито: чи сам `conformance.json` колись фізично розділиться на `language-core.json` / `stdlib.json` / `symbolic.json` за рівнем, тепер коли теги вже існують у `conformance-tier-map.json` незалежно від форми самого файлу; і саме питання ратифікації — цей документ і `my-lisp-constitution.json` лишаються чернетками до явної ратифікації, після якої конституція стане read-only, узгоджено з обіцянкою незмінності релізних тегів, яку проєкт уже практикує деінде (`docs/versioning.md`).
