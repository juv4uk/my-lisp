# Uniform Conceptual & Pāṇinian Audit of the Seven McCarthy Primitives
# Єдиний концептуальний та паніньянський аудит семи примітивів Маккарті

**Date:** 2026-09-06
**Status:** In Progress / Research Foundation for ADR-004
**Methodological Directive:**
1. Do not start with mechanical translation from English names.
2. Formulate the **observable semantic act** for each primitive.
3. Identify the **Indian conceptual witness** (Vyākaraṇa / Nyāya / Mīmāṃsā).
4. Apply **Pāṇinian morphological & terminological analysis** (root, derivation, saṃjñā discipline).
5. Review Sanskrit and Ukrainian surface candidates simultaneously.
6. Acknowledge semantic overclaims and risks.
7. Treat all 7 surface names as *accepted candidates pending uniform audit*, not immutable definitions.

---

## The Master Matrix / Головна матриця

| Canon | Semantic Act | Indian Conceptual Witness | Pāṇinian Analysis | Sanskrit Candidate | Ukrainian Candidate | Semantic Risk / Notes |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| `PRIM_QUOTE` | Suppress ordinary evaluation; treat expression as its own form | AS 1.1.68 `svaṃ rūpaṃ śabdasyāśabdasaṃjñā` | Sva-rūpa (bahuvrīhi compound: sva + rūpa) | `svarūpa` (स्वरूप) | **власна-форма** / **як-є** (дослівно?) | Risk: Philosophical overclaim ("essential nature"). Grammatical reading is exact. |
| `PRIM_ATOM` | Predicate testing if structural item is atomic (non-pair / indivisible by projection) | VS 7.1.10–11, NS 2.2.24, TS §25 (`aṇu` vs `mahat`) | Nominal base (prātipadika, avyutpanna) | `aṇu` (अणु) | **атом** | Risk: Vaiśeṣika physical ontology / relative size vs McCarthy discrete structural boolean. |
| `PRIM_EQ` | Atom identity predicate (structural/symbolic sameness, not numerical equality) | Tarkasaṅgraha §80 (`tādātmya` vs `anyonyābhāva`) | Nañ-compound (`a-` + `bheda` from √bhid) | `abheda` (अभेद) | **тотожне** | Risk: Conflating equality of value (`sama`) with ontological identity (`abheda`). |
| `PRIM_CAR` | Extraction/projection of first component of a pair | VS 4.2.9 (`āditva`), Mīmāṃsāsūtra (`prathama` / `pūrva`) | Nominal base / ordinal base | `ādi` (आदि) / `prathama` (प्रथम) | **перше** | Risk: Sequence/temporal bias vs abstract left/first coordinate projection. |
| `PRIM_CDR` | Extraction/projection of remaining component after first | NS 1.1.5 (`śeṣavat` inference), Mīmāṃsā `śeṣa` | kṛt-derived nominal from √śiṣ (gaṇa 7) | `śeṣa` (शेष) | **решта** | Risk: List/tail bias vs general pair second coordinate projection. |
| `PRIM_CONS` | Constructor: synthesize pair from two values | TS §27 (`saṃyuktavyavahāra`), √yuj (gaṇa 7) | Verbal base + upasarga (sam + √yuj) or kṛt-ana (`saṃyojana`) | `saṃyuj` (संयुज्) / `saṃyojana` (संयोजन) | **сполучити** | Risk: State of relation (`saṃyoga`) vs dynamic constructive act. |
| `PRIM_COND` | Ordered conditional selection: scan branches in sequence, evaluate first satisfying | Mīmāṃsā `krama`/`prathama`, AS 1.4.2 `vipratiṣedhe paraṃ kāryam` | Ordered selection / sequence resolution | `anukrama` (अनुक्रम) / `krama` (क्रम) | **вибір-за-порядком** / **за-умовою** | Risk: False equivalence with unordered `if` or disjunctive `vikalpa`. |

---

## Detailed Analyses per Primitive

### 1. PRIM_QUOTE
- **Observable Semantic Act:** An evaluator normally takes an expression and executes it as instructions. `PRIM_QUOTE` disables this transition: `eval(quote(e), env) = e`.
- **Indian Witness:** Aṣṭādhyāyī 1.1.68 (*svaṃ rūpaṃ śabdasyāśabdasaṃjñā*). A word denotes its own formal shape unless defined as an operational technical designation (`saṃjñā`).
- **Pāṇini vs McCarthy:** 
  - Evaluation in Lisp evaluates a symbol to its bound value, like `saṃjñā` resolving to its class.
  - `quote` reverts the evaluation mode to `svaṃ rūpam` (the symbol as its literal token).
- **Candidates:**
  - Sanskrit: `svarūpa` (स्वरूप).
  - Ukrainian: `власна-форма` (precise conceptual reflection), `як-є` (natural vernacular), or `дослівно` (literal/verbatim).

### 2. PRIM_ATOM
- **Observable Semantic Act:** Binary predicate: returns true if the argument is a primitive indivisible element (symbol, number) and false if it is a compound pair `(A . B)`.
- **Indian Witness:** Vaiśeṣika-sūtra 7.1.10–11; Nyāyasūtra 2.2.24 (*aṇunityatvāt*).
- **Pāṇinian Analysis:** `aṇu` functions as an un-decomposed base denoting the minimal limit of structure.
- **Candidates:**
  - Sanskrit: `aṇu` (अणु).
  - Ukrainian: `атом` (well-grounded in epistemology and computing; captures "indivisible token").

### 3. PRIM_EQ
- **Observable Semantic Act:** Identity test over atoms: `eq[x, y] = true` iff `x` and `y` designate the exact same atomic symbol/token.
- **Indian Witness:** Tarkasaṅgraha §80 (`tādātmyasaṃbandhāvacchinna-pratiyogitāko 'nyonyābhāvaḥ`). The identity relation is `tādātmya`. Its non-difference is `abheda`.
- **Pāṇinian Analysis:** `abheda` is a strict `nañ`-tatpuruṣa (`na bhedaḥ = abhedaḥ`). It negates difference, establishing identity.
- **Candidates:**
  - Sanskrit: `abheda` (अभेद). (Rejects `sama` as quantitative/qualitative similarity).
  - Ukrainian: `тотожне` (rigorous identity; distinguishes from arithmetic `=`).

### 4. PRIM_CAR & 5. PRIM_CDR (The Coordinate Projections)
- **Observable Semantic Act:** 
  - `car[(A . B)] = A` (projection of first element).
  - `cdr[(A . B)] = B` (projection of second/remaining element).
- **Indian Witness:** 
  - CAR: `āditva` (primacy in ordering, VS 4.2.9); `prathama` (first in sequential rule application, Mīmāṃsā).
  - CDR: `śeṣa` (Nyāyasūtra 1.1.5 `śeṣavat` — inference from the residual remainder).
- **Pāṇinian Analysis:** 
  - `ādi` is a nominal base; `śeṣa` is a regular kṛt nominal from `√śiṣ` (to leave behind).
- **Structural Algebra:**
  - Ukrainian: `перше` / `решта`.
  - Sanskrit: `ādi` / `śeṣa`.

### 6. PRIM_CONS
- **Observable Semantic Act:** Dyadic constructor: takes value `x` and value `y` and allocates/constructs an ordered pair `(x . y)`.
- **Indian Witness:** Tarkasaṅgraha §27 (`saṃyukta-vyavahāra-hetuḥ saṃyogaḥ`).
- **Pāṇinian Analysis:** Root `√yuj` (gaṇa 7: `yunakti` / `yuṅkte`) + upasarga `sam-`.
  - Form `saṃyuj` (verbal root) expresses the action of conjoining.
  - Form `saṃyojana` (kṛt nominal with suffix `-ana`) expresses the constructive act.
- **Candidates:**
  - Sanskrit: `saṃyuj` (compact action symbol) or `saṃyojana` (action noun).
  - Ukrainian: `сполучити` (action verb) or `сполука` (result). `сполучити` keeps operational alignment.

### 7. PRIM_COND
- **Observable Semantic Act:** Evaluates list of clauses `((p1 e1) (p2 e2) ...)`. Finds the *first* `pi` whose evaluation is non-nil, and evaluates and returns its corresponding `ei`.
- **Indian Witness:** 
  - Mīmāṃsāsūtra (rule priority through `pāṭha-krama` / textual order of execution).
  - Aṣṭādhyāyī 1.4.2 (*vipratiṣedhe paraṃ kāryam*) and general Paribhāṣā priority hierarchies (where rules are scanned in a specific precedence order).
  - Contrasted with `vikalpa` (which represents unordered optionality/free choice).
- **Pāṇinian Analysis:** 
  - `krama` (order, succession from `√kram`).
  - `anukrama` (ordered sequence, step-by-step resolution).
- **Candidates:**
  - Sanskrit: `anukrama` (अनुक्रम) or `krama` (क्रम).
  - Ukrainian: `вибір` (too unordered), `за-умовою` (conditional), or `вибір-за-порядком`.

