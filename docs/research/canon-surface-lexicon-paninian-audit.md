# Research: Pāṇinian Morphology Audit for CANON Surface Lexicons
# Дослідження: Аудит паніньянської морфології для поверхневих лексиконів CANON

**Status:** Research / Methodology
**Date:** 2026-09-06
**Context:** ADR-004 §4.1 — Sanskrit surface layer
**Related repos:** `my-lisp-panini`, `shiva-sutras`, `sanskritworld_texts`

---

## Мета / Goal

Санскритські surface signs у CANON не перевіряються лише за Monier-Williams.
Вони проходять Pāṇinian morphological audit:

```text
PRIM_* semantic identity
        ↓
root/concept candidate (dhātu or nominal stem)
        ↓
Pāṇinian morphology witness
  - gaṇa, pada, seṭ/aniṭ status (from my-lisp-panini registry)
  - kṛt suffix if action-noun form needed
  - prefix (upasarga) legitimacy
        ↓
natural Sanskrit technical form
  - does the form occur in primary texts? (sanskritworld_texts)
  - what register? (technical/philosophical/ordinary)
        ↓
surface admission decision
```

**Критерій допуску (4 критерії з ADR-004 §4.1):**
1. Не суперечить semantic contract PRIM_*.
2. Дає правильну людську інтуїцію.
3. Природна в Sanskrit — тобто морфологічно коректна за Pāṇini.
4. Не вводить нової семантики.

**Честність над красою (CORE-RULE з my-lisp-panini):**
Якщо форма морфологічно некоректна або незасвідчена — так і записуємо.
`UNVERIFIED` або `FORM-UNCERTAIN` — чесніше за красиву брехню.

---

## Розшарування походження (4 layers, з my-lisp-panini)

Кожна форма позначається рівнем:

- `[PANINI]` — безпосередньо з Aṣṭādhyāyī/Dhātupāṭha
- `[SCHOLARLY]` — лексикографічна традиція (Monier-Williams, commentators)
- `[PRIMARY TEXT]` — засвідчено в конкретному тексті (з репо sanskritworld_texts)
- `[CANON SURFACE]` — рішення для my-lisp (може бути менш ніж [PANINI] — це чесно)

---

## Audit: PRIM_ATOM → `aṇu / अणु`

**Dhātu source:** *не* є dhātu-похідним. `aṇu` — nominal stem.

**[PANINI]:**
- `aṇu` — primary adjective/noun, listed in nominal paradigms.
- Etymologically from `√an` (to breathe) or considered un-derived (`avyutpanna`).
- Not in Dhātupāṭha as a root; the nominal form is primary.

**[PRIMARY TEXT]:**
- Vaiśeṣika-sūtra 7.1.10: `ato viparītam aṇu` — «opposite of mahat is aṇu»
- Vaiśeṣika-sūtra 7.1.11: `aṇu mahad iti tasmin viśeṣabhāvāt viśeṣābhāvāc ca`
- Nyāyasūtra 2.2.24: `na, aṇunityatvāt` — «no, because of the eternity of aṇu»
- Tarkasaṅgraha §25: `aṇu mahad dīrghaṃ hrasvaṃ ceti` — four size-categories

**Morphological note:**
`aṇu` used as standalone term (not prefix `aṇu-` in compounds) gives:
- As predicate: `(aṇu x)` — «is x atomic/minute?»
- As noun: the class of non-composite objects

**Semantic gap vs McCarthy:**
`aṇu` in Vaiśeṣika = opposite of `mahat` (great) — a *relative* size category.
McCarthy `atom` = binary structural predicate (cannot be car/cdr-ed).
Gap exists but is acceptable under Surface Correspondence Principle:
the intuition «not further decomposable» is shared; the formal definition differs.

**[CANON SURFACE] decision:** `aṇu` ✅ ADMITTED
Rejected: `paramāṇu` (too loaded with Vaiśeṣika physical ontology),
`niravayava` (morphologically correct but too definitional — defines the primitive).

---

## Audit: PRIM_CAR → `ādi / आदि`

**[PANINI]:**
- `ādi` — from prefix `ā-` + `√dā` (to give) or alternatively from `ā- + √i` (to go
  toward). More precisely treated as `ā` + `di` (the locative of `√dā`? Disputed).
- In standard Pāṇinian usage: appears as a *prātipadika* (nominal base).
- As standalone noun: «beginning, first element».
- As suffix in compounds: «beginning with, et cetera» (e.g., `rāmādi`).

**[PRIMARY TEXT]:**
- VS 4.2.9: `saṃjñāyā āditvāt` — «because of being the beginning/first of the designation»
- NS 2.2.13: `ādimatvāt aindriyakatvāt kṛtakavat upacārāt ca`
  `ādimatva` = property of having a beginning/first

**Morphological note:**
As standalone function name `(ādi x)`, the et-cetera suffix reading cannot
activate — that reading requires a compound context (`rāma-ādi`).
Standalone `ādi` = «the first, the beginning».

**[CANON SURFACE] decision:** `ādi` ✅ ADMITTED
With the note: standalone form avoids the et-cetera compound ambiguity.

---

## Audit: PRIM_CDR → `śeṣa / शेष`

**[PANINI]:**
- From `√śiṣ` (class 7, rudhādi) = «to leave, to remain over».
- `śeṣa` = kṛt-derived nominal: «that which remains» (suffix `-a` from the root).
- Standard kṛt formation: `śiṣ + a` → `śeṣa` (with guṇa of the root vowel).

**[SCHOLARLY]:** Monier-Williams: «remainder, rest, residue, remnant».

**[PRIMARY TEXT]:**
- NS 1.1.5: `pūrvavat śeṣavat sāmānyatodṛṣṭaṃ ca`
  `śeṣavat` = «inference from the remainder» — one of THREE canonical inference types.
  `śeṣa` is a technical logical term for «what remains after exclusion of one element».

**Morphological note:**
`śeṣa` is a fully regular kṛt-nominal from `√śiṣ`. The form is:
- Morphologically clean (attested root + standard -a suffix).
- Semantically: «that which is left after removing the first» = exact CDR semantics.

**[CANON SURFACE] decision:** `śeṣa` ✅✅ ADMITTED (strongest of the triad)
Primary text usage in technical logical context directly mirrors CDR operation.

---

## Audit: PRIM_EQ → `abheda / अभेद`

**[PANINI]:**
- `a-` (negative prefix, Pāṇini 6.3.73 `nañ`) + `bheda` (nominal from `√bhid`,
  class 7 = «to split, cleave, differentiate»).
- `bheda` = kṛt-nominal: «splitting, difference, distinction».
- `abheda` = «non-difference, non-distinction, identity».
- Morphologically: standard `nañ`-compound. `a + bheda` → `abheda`.

**[SCHOLARLY]:** Monier-Williams: «non-difference, identity».

**[PRIMARY TEXT]:**
- Tarkasaṅgraha §80: `tādātmyasaṃbandhāvacchinna-pratiyogitāko 'nyonyābhāvaḥ`
  `tādātmya` = «sameness of essence» — the identity relation in Nyāya.
  `anyonyābhāva` = mutual non-existence (the negation of identity), defined through `tādātmya`.
  — `abheda` is the direct positive correspondent of `tādātmya`.

**Semantic note:**
McCarthy `eq[x;y]` = T iff x and y are the *same atom*.
`abheda` = «non-difference» = the two things are not distinct = identity.
This is precise: not «equal in value», not «similar», but «not-different».
Preferred over:
- `sama` (equal — too broad, covers quantitative/qualitative equality)
- `tulya` (equal by measure — quantity, not identity)
- `tādātmya` (correct but compound, two words)

**[CANON SURFACE] decision:** `abheda` ✅ ADMITTED
Morphologically regular nañ-compound. Primary text semantic field confirmed.

---

## Audit: PRIM_CONS → `saṃyuj / संयुज्`

**[PANINI] — dhātu registry (my-lisp-panini):**
```
canonical: yuj
gaṇa: 7 (rudhādi — infix nasal vikaraṇa: yu-NA-j)
pada: ubhayapada
seṭ: seṭ
traditional_meaning: «з'єднувати, запрягати»
```
- `sam-` (upasarga = verbal prefix, «together/completely»).
- `saṃ + yuj` → `saṃyuj` — root form with prefix.
- This is the root form (dhātu + upasarga), not yet a derived surface word.
- As infinitive: `saṃyoktum`; as action noun (kṛt -a, ghañ): `saṃyoga`; as verbal
  noun (kṛt -ana): `saṃyojana`.

**[PRIMARY TEXT]:**
- Tarkasaṅgraha §27: `saṃyuktavyavahārahetuḥ saṃyogaḥ`
  `saṃyoga` = «cause of the usage/treatment of things as joined».
  Note: `saṃyoga` is a *guṇa* (quality) — the *state* of being joined, not the *action*.

**Morphological question:**
`saṃyuj` is the root+prefix form. In Pāṇinian usage, functions as:
- verb stem: `saṃyujyate` (passive), `saṃyunakti` (class 7, active)
- Lisp function names typically match action-nouns or imperative-like forms

More natural surface forms from `√yuj` + `sam-`:
- `saṃyojana` (kṛt -ana suffix) = «act of joining» — action noun, most natural
- `saṃyoga` = «state of being joined» — less precise for a constructor
- `saṃyuj` = root form — somewhat technical but compact and action-oriented

**Assessment:**
`saṃyuj` (root form) is used here as a compact action-verb sign — analogous
to how `cons` is itself a truncated form, not a full word. Under Surface
Correspondence Principle this is acceptable: `saṃyuj` does not define the primitive,
it points toward the *action* of joining/pairing.

**[CANON SURFACE] decision:** `saṃyuj` ✅ ADMITTED
Note: `saṃyojana` (action noun) is an equally valid alternative with cleaner
nominal morphology. Both point to the same identity. Decision: prefer `saṃyuj`
for compactness and verb-action orientation, consistent with naming the *operation*.

---

## Open: PRIM_QUOTE — Research Path

**Semantic contract:** `(quote expr)` — return `expr` unevaluated, as-is.

**Core question:** not «what is the Sanskrit word for quotation?» but
«what Sanskrit root/concept expresses 'in this very form / without transformation'?»

**Candidate dhātu in my-lisp-panini registry:**
```
canonical: vac    (adādi, gaṇa 2, parasmaipada, aniṭ)
meaning: «говорити / to speak»
```
- `vac` → `vāc` (speech, word as stated) — the feminine action-noun.
- `yathāvac` or `yathā-ukta` — «as stated» — a possible direction.
- But `vac` points to utterance, not to non-evaluation.

**Better candidate: `tathā / yathā`** family:
- `yathā` = «as, in the manner of» — relational, not a primitive name.

**Stronger candidate: `avikṛta` (अविकृत)**
- `a-` + `vikṛta` (kta-participle from `vi-√kṛ` = to modify/transform)
- `avikṛta` = «unmodified, untransformed, as-is»
- Morphologically: standard nañ + kta-participle. Clean.
- Semantic fit: `(quote x)` returns x *avikṛta* — x as-is, without transformation.

**This is a CANDIDATE, not a decision.** Needs primary text evidence.
Research path: find `avikṛta` in a philosophical context meaning «returned in its
original form without processing» in `sanskritworld_texts`.

Ukrainian path: `дослівно` (literally / word-for-word) — strong candidate.
«(дослівно x)» = «take x literally, don't process it» — correct intuition.

---

## Open: PRIM_COND — Research Path

**Semantic contract:**
```
(cond ((p₁) e₁) ((p₂) e₂) ... ((pₙ) eₙ))
→ eᵢ where pᵢ is the first true predicate
```

**Core question:** not «if», but «select the first true branch».

**Why `yadi` (यदि, «if») is weak:**
`yadi` = conditional conjunction = introduces a single condition.
`cond` is a *selection* over a sequence — closer to discriminative choice.

**Candidate concepts:**
- `vibhāga` (विभाग) = division, partition, selection — from `vi + √bhaj`
- `vikalpa` (विकल्प) = alternative, option — from `vi + √kḷp`
  NS 1.1.44: `vikalpa` = disjunction / either-or option in Nyāya
- `nirṇaya` (निर्णय) = determination, decision — from `nir + √nī`
- `prakaraṇa` (प्रकरण) = context-based selection

**`vikalpa` is the strongest candidate:**
- Nyāyasūtra uses `vikalpa` for mutually exclusive alternatives: select one.
- `(vikalpa ...)` = «select from alternatives» — maps to cond's sequential selection.
- But: `vikalpa` in Pāṇini also means «optionality» in rules (the rule may or may not apply).
  Semantic overlap possible.

**Ukrainian path:** `якщо` is too single-conditional.
Candidates: `коли` (when — sequential), `залежно` (depending on), `вибір` (choice/selection).
`вибір` (choice) may be the cleanest: `(вибір ...)` = «make a choice from conditions».

**Research path:** find `vikalpa` in Nyāya context as «selection of first applicable
alternative» in `sanskritworld_texts`. Then compare with `vibhāga`.

---

## Summary / Підсумок

| Primitive | UK surface | UK status | SA surface | SA status | Pāṇinian form |
| :--- | :--- | :--- | :--- | :--- | :--- |
| `PRIM_ATOM` | `атом` | ✅ | `aṇu` | ✅ | nominal primary |
| `PRIM_EQ` | `тотожне` | ✅ | `abheda` | ✅ | nañ-compound (a+bheda) |
| `PRIM_CAR` | `перше` | ✅ | `ādi` | ✅ | nominal primary |
| `PRIM_CDR` | `решта` | ✅ | `śeṣa` | ✅ | kṛt-a from √śiṣ |
| `PRIM_CONS` | `сполучити` | ✅ | `saṃyuj` | ✅ | root+upasarga (√yuj+sam) |
| `PRIM_QUOTE` | `дослівно` | 🟡 candidate | `avikṛta`? | 🟡 unverified | kta-participle from vi-√kṛ |
| `PRIM_COND` | `вибір`? | 🟡 candidate | `vikalpa`? | 🟡 unverified | vi+√kḷp nominal |

**Sources used:**
- `my-lisp-panini/panini/registry/dhatu/yuj.yaml` — √yuj gaṇa 7, ubhayapada
- `my-lisp-panini/panini/registry/dhatu/vac.yaml` — √vac gaṇa 2, aniṭ
- `my-lisp-panini/panini/sastra/pratyaya.md` — kṛt suffix taxonomy
- `my-lisp-panini/panini/sastra/dhatu.md` — dhātu definition and 4-layer model
- `sanskritworld_texts/shastra/philosophy/vaisheshika/vaisheShikasUtra.txt` — VS 4.2.9, 7.1.10-11
- `sanskritworld_texts/shastra/philosophy/nyaya/nyAyasUtra.txt` — NS 1.1.5, 2.2.24
- `sanskritworld_texts/shastra/philosophy/nyaya/tarkasaNgraha.txt` — §25, §27, §80
