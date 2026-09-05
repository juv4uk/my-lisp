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

## Open: PRIM_QUOTE — Research Path (calibrated 2026-09-06)

**Semantic contract:**
```
(quote expr) → expr    — returned unevaluated, as the form itself
```

**What QUOTE does NOT mean:**
```
QUOTE ≠ immutability       (the object may be mutated later)
QUOTE ≠ copying unchanged  (no copy is made)
QUOTE ≠ preservation in general sense
```

`(quote x)` specifically suppresses the *evaluation mode* applied to x:
«do not interpret x as an instruction; treat x as the form itself».

**Why `avikṛta` is REJECTED:**
`avikṛta` = «unmodified/untransformed» — speaks about the *state* of the object,
not about the *evaluation suppression*. It would read as «x is unmodified»,
which overclaims: quote does not guarantee object immutability, it only
suppresses evaluation. Under criterion 4 (must not introduce new semantics):
REJECTED.

**Correct semantic axis:**
Not «preserved/unchanged», but: «taken as named / as the form stated / as-is-in-itself».

**Candidate investigation — `vac` family (from my-lisp-panini registry):**
```
√vac  gaṇa 2, parasmaipada, aniṭ
traditional_meaning: «говорити / to speak»
kta-participle: ukta (उक्त) = «spoken, stated, named»
ghañ nominal: vāka / vāc (वाच्) = «speech, utterance»
```

**`ukta / उक्त` — kta-participle from √vac:**
- Vaiśeṣika-sūtra 7.1.1: `uktā guṇāḥ` = «the guṇas [thus] stated/named»
  `ukta` = «what has been said / what has been named» — the form as named.
- Morphologically: regular kta-passive participle, fully attested.
- Semantic reading: `(ukta x)` = «x as stated / x as named» — the form x, not its value.
- Risk: `ukta` implies something *already said*, pointing backward. Quote is prospective.

**`svarūpa / स्वरूप` — «own form / as-it-is-in-itself»:**
- Tarkasaṅgraha §56: `svarūpāsiddha` — «unestablished in its own form» — technical
  logical term for «failing to be what it presents itself as».
- `svarūpa` = `sva` (own) + `rūpa` (form) — «the thing's own form, as-it-is».
- Not from √vac lineage, but directly captures «treat x in its own form, not
  as an instruction to evaluate».
- `(svarūpa x)` = «return x in its own form» — semantically very close to quote.
- Risk: may read as «the nature/essence of x» in philosophical contexts.

**`svārtha / स्वार्थ` — «its own meaning/referent»:**
- Tarkasaṅgraha §48: `svārtham` — «for its own sake / its own referent».
- But this points to what an expression *means*, not to the form itself. Too indirect.

**Current state: OPEN — three candidates under investigation.**
```
ukta     (uktā guṇāḥ — VS 7.1.1)    — kta form, «as stated»
svarūpa  (svarūpāsiddha — TS §56)    — «in its own form / as-is»
[unknown] — vac-family further needed
```
Neither is yet admitted. Need: primary text evidence for the reading
«return this in its own form without evaluation/transformation».

---

## Open: PRIM_COND — Research Path (calibrated 2026-09-06)

**Semantic contract:**
```
(cond ((p₁) e₁) ((p₂) e₂) ... ((pₙ) eₙ))
→ eᵢ where pᵢ is the FIRST true predicate (ordered selection)
```

**The crucial structural feature:** `cond` is not just «choice» or «alternative» —
it is **ordered sequential scanning**: evaluate p₁, if true return e₁; else
evaluate p₂, etc. Order matters. Termination on first success.

**Why `vikalpa` is INSUFFICIENT:**
- Nyāyasūtra `vikalpa` = mutually exclusive alternatives (either-or disjunction).
- In Pāṇini, `vikalpa` = optional rule application (may or may not apply) — unordered.
- Neither captures the *ordered* and *first-true* structure of `cond`.
- `vikalpa` admits all alternatives as coequal; `cond` has a priority order.
- STATUS: `vikalpa` REJECTED for now. Weaker than needed.

**Candidates still open:**
- `krama` (क्रम) = sequence, order — from `√kram` (to step). Captures order, not selection.
- `kramavikalpа` — compound? Non-standard, not in primary texts.
- `anukrama` (अनुक्रम) = sequential order — from `anu + √kram`.
  Used in technical Sanskrit for «in order, step by step».
- `nirṇaya` (निर्णय) = determination, decision — from `nir + √nī`.
  NS: `nirṇaya` = arriving at a definite conclusion. Captures the *resolution* but
  not the sequential scanning.

**The right question for further research:**
Is there a Sanskrit technical term for «evaluate in order, take the first that applies»?
This is closer to the Mīmāṃsā method of rule application (`prāptapratipratipatti`?)
than to Nyāya `vikalpa`.

**Ukrainian candidates (open):**
- `якщо` — single conditional, too weak for sequential selection. ❌
- `вибір` — choice, but implies unordered selection.
- `за умовою` — prepositional phrase, not a natural function name.
- `коли перше` — too verbose.
- No clear winner yet. **OPEN.**

**Research path:** examine Mīmāṃsā and Nyāya for terms describing
ordered rule application / first-applicable-rule semantics.
Check `sanskritworld_texts/shastra/philosophy/mimamsa/`.

---

## Grammatical Form Policy / Правило граматичної форми

*(Added 2026-09-06 after owner calibration)*

Sanskrit surface names for PRIM_* **may be of any grammatical class**
provided the form is natural and readable as a Lisp operator name.
No single part of speech is required across all seven.

```text
Criterion E (chosen): natural as a technical operator in its language.

aṇu    — nominal adjective/noun (not dhātu-derived)
ādi    — nominal (primary)
śeṣa   — kṛt-a nominal from √śiṣ
abheda — nañ-compound nominal
saṃyuj — dhātu + upasarga (root-form as compact action sign)
```

**Calibration note on `saṃyuj`:**
`saṃyuj` is a verbal root-form (dhātu + upasarga), not a nominal.
This is explicitly noted and accepted: `cons` itself is a compressed operational
label, not a natural-language noun. `saṃyuj` as a surface sign is consistent
with the Surface Correspondence Principle — it names the *action* without
defining the primitive. Alternative nominal forms:
- `saṃyojana` (kṛt -ana = action noun, «act of joining») — equally valid, more nominal
- `saṃyoga` (ghañ = state noun, «state of being joined») — REJECTED (state ≠ action)
Decision: `saṃyuj` retained as compact action-oriented sign.

**Pāṇinian role:** Pāṇini **verifies the form** (morphological legitimacy,
root attestation, suffix regularity). Pāṇini does NOT determine what PRIM_*
means. The authority chain:

```text
CANON
  ↓
semantic invariant (owner authority)
  ↓
surface candidate
  ↓
Pāṇinian morphological witness  ← verifies form only
  ↓
lexical/textual witness          ← confirms register and usage
  ↓
owner admission
```

---

## Summary / Підсумок (calibrated 2026-09-06)

| Primitive | UK surface | UK status | SA surface | SA status | Pāṇinian form | Notes |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| `PRIM_ATOM` | `атом` | ✅ | `aṇu` | ✅ | nominal primary | |
| `PRIM_EQ` | `тотожне` | ✅ | `abheda` | ✅ | nañ + bheda | semantic caution documented |
| `PRIM_CAR` | `перше` | ✅ | `ādi` | ✅ | nominal primary | standalone only |
| `PRIM_CDR` | `решта` | ✅ | `śeṣa` | ✅ | kṛt-a from √śiṣ | strongest of five |
| `PRIM_CONS` | `сполучити` | ✅ | `saṃyuj` | ✅ | √yuj+sam root form | form policy §above |
| `PRIM_QUOTE` | `дослівно` | 🟡 | `ukta`/`svarūpa`? | 🟡 open | — | avikṛta REJECTED |
| `PRIM_COND` | ? | 🟡 | ? | 🟡 open | — | vikalpa REJECTED |

**Sources used:**
- `my-lisp-panini/panini/registry/dhatu/yuj.yaml` — √yuj gaṇa 7, ubhayapada
- `my-lisp-panini/panini/registry/dhatu/vac.yaml` — √vac gaṇa 2, aniṭ
- `my-lisp-panini/panini/sastra/pratyaya.md` — kṛt suffix taxonomy
- `my-lisp-panini/panini/sastra/dhatu.md` — dhātu definition and 4-layer model
- `sanskritworld_texts/shastra/philosophy/vaisheshika/vaisheShikasUtra.txt` — VS 4.2.9, 7.1.1, 7.1.10-11
- `sanskritworld_texts/shastra/philosophy/nyaya/nyAyasUtra.txt` — NS 1.1.5, 2.2.24
- `sanskritworld_texts/shastra/philosophy/nyaya/tarkasaNgraha.txt` — §25, §27, §56, §80


---

## QUOTE CONCEPT AUDIT — Deep Pass (2026-09-06)

### 1. Observable Contract of PRIM_QUOTE

```text
(quote expr) → expr

Contract:
- expr is NOT evaluated as a computation instruction
- expr is returned AS THE FORM ITSELF
- the evaluation mode is suppressed, not the object's content

What this IS:
  suppression of evaluation mode → take-as-form

What this IS NOT:
  immutability of object
  copying
  preservation of state
  protection from future modification
```

### 2. Required Distinctions / Необхідні розрізнення

For a Sanskrit surface sign to be valid, the tradition must have a term that
targets the right distinction in the following map:

```text
expression
    │
    ├── (A) as instruction / as computation target
    │         → normal eval mode
    │
    └── (B) as the form itself / as the named thing
              → QUOTE mode
```

The sign must point to (B) without importing unwanted meanings from adjacent concepts.

### 3. Candidate Audit

---

#### `ukta / उक्त` — kta-participle from √vac

**ROOT:** √vac, gaṇa 2, aniṭ (my-lisp-panini registry: unverified).
**FORM:** kta-passive participle: `vakta → ukta` (irregular strong → weak).
**MEANING:** «spoken, stated, named» — the form as uttered.

**PRIMARY WITNESS:**
- VS 7.1.1: `uktā guṇāḥ` — «the guṇas [thus] stated» — past reference.
- Tarkasaṅgraha §59: `āptavākyaṃ śabdaḥ` — word/śabda as reliable utterance.

**MATCH TO QUOTE:**
`ukta` captures «what has been said» — i.e., the form as stated, not its referent.
Semantic reading: `(ukta x)` = «x as stated, as it was uttered».
The form, not its evaluated content.

**OVERCLAIM RISK:**
`ukta` is retrospective — it points to something *already said*.
`(quote x)` is prospective: «do not evaluate the following x».
The temporal direction differs. Moderate risk.

**VERDICT:** 🟡 Directionally correct, temporal mismatch.

---

#### `svarūpa / स्वरूप` — «own form»

**FORM:** `sva` (own) + `rūpa` (form) — bahuvrihi compound.
**MEANING:** «own form, the form as it is in itself».

**PRIMARY WITNESS:**
- Tarkasaṅgraha §56: `svarūpāsiddha` — «unestablished in its own form» —
  logical fallacy where the *own form* of the proposed reason is not established.
  `svarūpa` here = «what the thing IS in its own right, before any predication».

**KEY WITNESS — Aṣṭādhyāyī 1.1.68:**
```
svaṃ rūpaṃ śabdasyāśabdasaṃjñā
```
«A word's own form [denotes itself], unless it functions as a non-phonetic
technical term (aśabdasaṃjñā).»

This sūtra establishes a **binary distinction within Pāṇini's grammar**:
```text
śabda used as saṃjñā (technical term)  → refers to its DEFINED CLASS
                                           (not to its phonetic shape)
śabda in svaṃ rūpam mode               → refers to ITSELF AS A FORM
                                           (not to what it usually means)
```

This is PRECISELY the distinction `(quote expr)` makes:
```text
expr in normal eval mode  → processed as instruction (like saṃjñā → class)
expr under QUOTE          → taken as its own form (like svaṃ rūpam mode)
```

**MATCH TO QUOTE:** Very strong. AS 1.1.68 is the primary witness.
The grammatical operation «take this word in its svaṃ rūpam» =
«do not treat it as pointing to a class / do not process its reference chain».
This maps directly to `(quote x)` = «take x as its own form, not as instruction».

**OVERCLAIM RISK:**
In philosophical contexts, `svarūpa` can mean «the essence/nature of X» —
which is a different claim. But in grammatical context (AS 1.1.68), the reading
is technical and precise: the phonetic form, not the class it names.
The risk is context-leakage from philosophy into grammar. Moderate, but the
AS 1.1.68 primary witness controls it.

**VERDICT:** ✅ **STRONGEST CANDIDATE.** Primary witness is AS 1.1.68 itself.

---

#### `saṃjñā / संज्ञा` — technical designation

**FORM:** `sam + jñā` (√jñā = to know, with sam-).
**MEANING:** Technical name/designation, a label that enables reference to a class.

**PRIMARY WITNESS:**
- AS 1.1.1: `vṛddhir ādaic` — «ā, ai, au are [called] vṛddhi» — saṃjñā-sūtra.
- Tarkasaṅgraha §58: `saṃjñā-saṃjñi-sambandha-jñānam upamitih` —
  «knowledge of the relation between the name (saṃjñā) and the named (saṃjñin)».
- AS 1.4.1: single saṃjñā per entity (conflict resolution).
- sarvam-hostile-review.md [TEXTUAL EVIDENCE]: saṃjñā *enables reference*, does not restrict.

**CRITICAL FINDING:**
`saṃjñā` is NOT a candidate for `QUOTE`. Rather, it reveals the architectural
intersection of CANON and QUOTE:

```text
CANON                               QUOTE (PRIM_QUOTE)
  │                                       │
identity ≠ spelling               form ≠ evaluation-of-form
  │                                       │
PRIM_ATOM is the semantic identity   (quote x) suppresses
behind 'atom'/'атом'/'aṇu'           the evaluation of x
  │                                       │
saṃjñā mechanism:                  svaṃ rūpam mechanism:
  label → class reference            form → form itself
  (technical term mode)              (own-form mode)
              │                           │
              └──────────┬────────────────┘
                         ↓
              AS 1.1.68 is the junction:
              the sūtra that distinguishes
              these two modes
```

**VERDICT for `saṃjñā` as QUOTE name:** ❌ NOT a surface sign for PRIM_QUOTE.
But `saṃjñā` reveals that AS 1.1.68's `svaṃ rūpam` is the **structural key**.

---

#### `abhidhā / abhidhāna / अभिधा, अभिधान`

**ROOT:** `abhi + dhā` (to place upon, to name).
**MEANING:** `abhidhāna` = «naming, denotation, the act of indicating»; `abhidhā` = «primary denotation».
Nyāya-Vaiśeṣika: `śakti` = denotative power; `abhidhā` = how a word refers to its primary meaning.

**PRIMARY WITNESS:**
- Tarkasaṅgraha §59: `śaktaṃ padam` — «a word [is] one that has [denotative] capacity».
  `abhidhā` is the *power of denotation*, not the form itself.

**MATCH TO QUOTE:**
`abhidhā` points to the *relationship between sign and referent* — the power of
denotation. Quote suppresses denotation, but is not *about* denotation.
`(abhidhā x)` would read as «the denotative aspect of x» — overclaims the
mechanism, not the form.

**VERDICT:** ❌ Wrong axis. `abhidhā` is about denotation; QUOTE suppresses evaluation.

---

#### `śabda / शब्द` — word/sound/utterance

**PRIMARY WITNESS:**
- Tarkasaṅgraha §59: `āptavākyaṃ śabdaḥ` — reliable testimony.
  `śabda` = word-as-testimony, word-as-pramāṇa (valid means of knowledge).
- §63: `vākyārthajñānaṃ śabdajñānam` — knowledge of sentence-meaning = śabda-knowledge.

**MATCH TO QUOTE:**
`śabda` in Nyāya is a pramāṇa (source of valid knowledge) — the opposite direction
from QUOTE. QUOTE *suppresses* the word from functioning as instruction. `śabda` is
about the word as a *source of knowledge*. Fundamentally misaligned.

**VERDICT:** ❌ Opposite direction.

---

### 4. Synthesis / Синтез

**AS 1.1.68 is the primary conceptual witness for PRIM_QUOTE.**

The sūtra `svaṃ rūpaṃ śabdasyāśabdasaṃjñā` establishes a binary:
```text
MODE A (normal): word functions as technical term → refers to class
MODE B (svaṃ rūpam): word taken in its own form → refers to itself

PRIM_QUOTE invokes MODE B on any expression:
(quote expr) → take expr in svaṃ rūpam, not as instruction.
```

**`svarūpa` is the strongest Sanskrit candidate** for PRIM_QUOTE surface sign:
- Morphologically regular compound (sva + rūpa).
- Primary witness: AS 1.1.68 (svaṃ rūpaṃ).
- Technical reading in grammar: «the form itself, not the class it refers to».
- Operational reading: «take this in its own form / without processing its reference».
- Risk: in philosophy, can mean «own nature/essence» — context-leakage. Moderate.

**`saṃjñā` reveals the intersection with CANON** but is NOT the surface sign.

**Intersection finding (new, 2026-09-06):**
AS 1.1.68 is the junction where CANON and QUOTE meet:
- CANON uses the saṃjñā mode (PRIM_ATOM is a technical label for a semantic identity).
- QUOTE uses the svaṃ rūpam mode (take the form, not its evaluation).
Both live under the same sūtra, on opposite sides of its distinction.

**Current admitted status:** `svarūpa` = CANDIDATE UNDER REVIEW, not yet admitted.
Needs: owner ratification. The AS 1.1.68 witness is strong.
**`ukta`, `abhidhā`, `śabda`, `avikṛta`:** all REJECTED.
