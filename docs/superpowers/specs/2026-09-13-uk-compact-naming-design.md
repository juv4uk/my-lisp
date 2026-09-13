# UK Compact Naming Design

## Status

Design for `my-lisp` issues #86 and #89. This document defines how the short Ukrainian surface `uk` relates to the full Ukrainian surface `ukr`.

No compact candidate in this document is semantic authority. `lib/surface/semantic-registry.wsm` remains the only spelling authority. A compact candidate becomes `uk stable` only after the promotion process in this document is satisfied.

## Goal

The Ukrainian pair has two different jobs:

```text
uk   = compact Ukrainian spelling, easy to type and intuitively decode
ukr  = full Ukrainian spelling, explicit and semantically descriptive
```

Both are peer spellings of the same numeric semantic ID. Neither spelling is semantic identity.

The compact surface is not a codebook. A Ukrainian-speaking programmer who has not memorized a table should be able to make a reasonable guess at the full wording and, more importantly, the operation's meaning.

## Non-goals

- do not create a second Ukrainian implementation;
- do not change semantic IDs;
- do not force every `uk` spelling to differ from `ukr`;
- do not shorten a clear short word merely to save one or two characters;
- do not admit arbitrary consonant codes or unexplained acronyms;
- do not promote compact candidates only because an author likes them;
- do not remove an old stable Ukrainian spelling during migration.

## Design choice

Three compacting strategies were considered.

### A. Pure root clipping

Example: `помножити -> множ`, `обчислити -> обчис`.

This is maximally systematic and keeps a visible lexical root. It becomes hard to read when several clipped roots are combined.

### B. Structural compaction plus conservative root clipping — selected

First remove grammatical filler and use a consistent object-first compact grammar; only then clip long roots when the result remains pronounceable and recoverable.

Examples:

```text
довжина-вектора              -> вектор-довж
перший-символ-тексту         -> текст-перш
встановити-елемент-вектора!  -> вектор-встан!
помножити                     -> множ
```

This gives useful shortening without turning names into opaque codes.

### C. Free semantic mnemonics/synonyms

Example: `елемент-списку-за-індексом -> за-номером`.

This may be intuitive semantically, but it is not reliably reverse-decodable to the full spelling. It is therefore **not** the default compact rule. A free synonym may exist only through a separately reviewed compatibility/convenience alias, not as an automatic `uk` compact derivation.

## Core rules

### 1. Full form first

Every compact candidate is judged against one explicit `ukr` full spelling. If the full wording is still disputed, the compact form cannot be ratified first.

### 2. Preserve recognizable Ukrainian roots

A compact lexical segment must preserve a recognizable prefix/root from the full Ukrainian word. Do not delete internal vowels merely to minimize length.

Preferred:

```text
помножити  -> множ
обчислити  -> обчис
підставити -> підст
уніфікувати -> уніф
```

Rejected:

```text
помножити -> пмж
обчислити -> бчс
підставити -> пдст
```

A four-letter visible root is a default lower bound for clipping. Shorter forms require explicit evidence from the blind-decoding test.

### 3. Prefer structural shortening before clipping

For multiword names, remove grammatical filler and use semantic order before clipping every word.

Preferred pattern for object operations:

```text
<object>-<operation-or-property>
```

Examples:

```text
довжина-тексту       -> текст-довж
довжина-вектора      -> вектор-довж
отримати-з-карти     -> карта-отрим
вставити-в-карту     -> карта-встав
```

Words such as `у`, `із`, `за`, `для`, `від`, `до` are removed when their relation is recoverable from the compact pattern. They remain when removing them changes or obscures meaning.

### 4. Keep already-good short words whole

A full word that is already short, common, and specific should normally stay unchanged:

```text
атом?
список
пара
корінь
остача
частка
намір?
доказ?
нехай
```

Compactness is not a contest to minimize bytes.

### 5. Predicates keep `?`; mutation keeps `!`

The suffix is semantic information and is never removed during compaction.

```text
твердження? -> тверд?
спостереження? -> спост?
встановити-елемент-вектора! -> вектор-встан!
```

### 6. No Latin keyboard switch

`uk` follows the same keyboard principle as `ukr`: public Ukrainian identifiers contain no ASCII Latin letters. Technical concepts must use Ukrainian-script words or remain unratified.

### 7. Avoid initialisms unless blind-tested

Forms such as `всч`, `нс`, `мс` are short but require prior decoding knowledge. They are not automatically acceptable merely because they already exist.

Prefer human-readable candidates such as `нано`, `мілі`, `час-коорд` only if the blind test confirms them.

### 8. One compact spelling cannot silently own two semantic IDs

Collision checks are numeric-ID based and fail closed. If two operations naturally compress to the same spelling, at least one candidate must be revised.

### 9. Two semantic chunks are preferred

One or two hyphen-separated semantic chunks are the target. Three are allowed when necessary. More than three normally means the result is no longer compact enough or the full form should simply remain.

### 10. `uk == ukr` is valid

If the full name is already compact and intuitive, the two columns may be identical. Artificial shortening is worse than equality.

## Promotion lifecycle

A compact spelling moves through these states:

```text
proposed
  -> mechanically-valid
  -> blind-test
  -> clear | revise | ambiguous | reject
  -> stable (only if clear and collision-free)
```

### Proposed

Stored outside semantic authority as candidate evidence keyed by semantic ID.

### Mechanically valid

Must pass:

- Cyrillic/no-Latin policy;
- no cross-ID collision;
- punctuation parity for `?`/`!`;
- shorter than or equal to `ukr`;
- root/structure rule classification present;
- no duplicate candidate ownership.

### Blind test

Issue #89 protocol is mandatory before promotion of a changed production `uk` spelling. Participants see only the compact form and provide:

1. expected full wording;
2. expected semantic action;
3. confidence.

Repository stores only anonymized aggregate outcomes.

### Stable

After a candidate is `clear`, `semantic-registry.wsm` receives the compact spelling as `(uk ... stable)`.

The previous stable `uk` spelling is retained on the same semantic row as a `compat ... compatibility-only` spelling. This preserves old source while the new compact surface becomes primary.

Example migration shape:

```text
before:
  (1031 ... (uk елемент-списку-за-індексом stable)
              (ukr елемент-списку-за-індексом stable) ...)

after ratification:
  (1031 ... (uk список-елем stable)
              (ukr елемент-списку-за-індексом stable)
              ...
              (compat елемент-списку-за-індексом compatibility-only))
```

No new semantic identity is created.

## Candidate evidence format

Compact proposals should live in a non-authoritative evidence file keyed by semantic ID. It must not duplicate semantic meaning or implementation.

Recommended schema:

```text
(uk-compact-candidates/1
  (1031 (candidate список-елем)
        (rule object-root)
        (status proposed))
  ...)
```

The audit generator joins the candidate to `ukr` through the numeric ID from `semantic-registry.wsm`. The candidate file therefore does not repeat the full wording and cannot become a second semantic registry.

## Reviewed example set

The following are **examples/candidates for blind testing, not production names**. They are deliberately drawn from different registry classes.

| ID | Full `ukr` wording | Compact candidate | Rule | Initial review |
|---:|---|---|---|---|
| 0002 | `атом?` | `атом?` | keep | already compact |
| 0004 | `сполучити` | `сполучити` | keep | shortening would not help |
| 0010 | `функція` | `функція` | keep | already compact |
| 0011 | `визначити` | `визначити` | keep | clipped root risks ambiguity |
| 1002 | `помножити` | `множ` | root | strong candidate |
| 1003 | `поділити` | `поділити` | keep | `діл` too short/ambiguous pre-pilot |
| 1004 | `модуль` | `модуль` | keep | already compact |
| 1007 | `остача` | `остача` | keep | already compact |
| 1008 | `частка` | `частка` | keep | already compact |
| 1009 | `корінь` | `корінь` | keep | already compact |
| 1026 | `числовий-буфер?` | `числ-буфер?` | root+keep | test readability |
| 1027 | `список` | `список` | keep | already compact |
| 1028 | `довжина` | `довжина` | keep | one common word; clipping unnecessary |
| 1029 | `приєднати` | `приєд` | root | strong candidate |
| 1031 | `елемент-списку-за-індексом` | `список-елем` | object-root | strong semantic recovery candidate |
| 1032 | `значення-у-списку?` | `список-має?` | object-property | semantic recovery test required |
| 1033 | `знайти-за-ключем` | `за-ключем` | structural | clear meaning, exact wording recovery uncertain |
| 0101 | `відобразити` | `відобр` | root | check confusion with noun `відображення` |
| 0102 | `відсіяти` | `відсів` | root | readable, grammatical-class shift acceptable only if test is clear |
| 0103 | `згорнути` | `згорт` | root | strong candidate |
| 1043 | `зчепити` | `зчеп` | root | strong candidate |
| 1044 | `довжина-тексту` | `текст-довж` | object-root | strong candidate |
| 1045 | `порожній-текст?` | `текст-порож?` | object-root | test clipped adjective readability |
| 1047 | `фрагмент-у-тексті?` | `текст-фрагм?` | object-root | test semantic direction |
| 1048 | `перший-символ-тексту` | `текст-перш` | object-root | test whether “first” implies character |
| 1049 | `решта-символів-тексту` | `текст-решта` | object-keep | strong candidate |
| 1051 | `символ-у-текст` | `симв-текст` | root-structural | test conversion direction |
| 1052 | `текст-у-символ` | `текст-симв` | structural-root | paired with 1051 |
| 1055 | `число-у-текст` | `число-текст` | structural | strong candidate |
| 1057 | `друкувати` | `друк` | root | strong candidate |
| 1059 | `прочитати` | `прочит` | root | strong candidate |
| 1060 | `прочитати-усе` | `прочит-усе` | root+keep | paired with 1059 |
| 1061 | `значення-у-текст` | `знач-текст` | root-structural | test `знач` readability |
| 1062 | `обчислити` | `обчис` | root | strong candidate |
| 1065 | `створити-вектор` | `вектор-нов` | object-root | semantic synonym/root mix; needs test |
| 1066 | `довжина-вектора` | `вектор-довж` | object-root | strong candidate |
| 1067 | `елемент-вектора` | `вектор-елем` | object-root | collision review with 1101 required |
| 1068 | `встановити-елемент-вектора!` | `вектор-встан!` | object-root | strong candidate if mutation meaning recovered |
| 1075 | `монотонний-час-у-наносекундах` | `моно-нано` | technical-root | high-risk; blind test mandatory |
| 1076 | `поточний-юнікс-час` | `час-юнікс` | structural | likely clear to technical users |
| 1084 | `назва-часового-поясу` | `пояс-назва` | object-keep | strong candidate |
| 1085 | `визначити-часовий-пояс` | `пояс-визн` | object-root | check ambiguity of `визн` |
| 1086 | `зміщення-часового-поясу-в-секундах` | `пояс-зсув` | object-synonym | strong semantic candidate, exact recovery lower |
| 1094 | `отримати-з-карти` | `карта-отрим` | object-root | strong candidate |
| 1095 | `вставити-в-карту` | `карта-встав` | object-root | strong candidate |
| 1096 | `ключ-у-карті?` | `карта-ключ?` | object-keep | strong candidate |
| 1108 | `пряме-виведення` | `прям-вивід` | root-root | test readability |
| 1115 | `пояснити-доведення` | `доказ-поясн` | object-root | terminology review needed |
| 1120 | `уніфікувати` | `уніф` | root | common technical root |
| 1121 | `логічна-змінна` | `лог-змінна` | root-keep | test whether `лог` is too broad |
| 1123 | `підставити` | `підст` | root | strong candidate |
| 1126 | `твердження?` | `тверд?` | root | test adjective confusion |
| 1127 | `зміст-твердження` | `тверд-зміст` | root-structural | paired with 1126 |
| 1128 | `стан-розгляду-твердження` | `тверд-стан` | root-structural | strong candidate if domain context is clear |
| 1132 | `спостереження?` | `спост?` | root | strong candidate |
| 1133 | `зміст-спостереження` | `спост-зміст` | root-structural | paired with 1132 |
| 1135 | `мета-наміру` | `намір-мета` | structural | strong candidate |
| 1138 | `створити-унікальний-символ` | `унік-симв` | root-root | test technical readability |
| 1142 | `нехай-послідовно` | `нехай*` | conventional syntax | already-established Lisp convention; still verify Ukrainian discoverability |
| 1147 | `запустити-процес` | `процес-запуск` | structural | strong candidate |
| 1148 | `прочитати-з-мережевого-з'єднання` | `мережа-прочит` | object-root | strong candidate |
| 1149 | `записати-у-мережеве-з'єднання` | `мережа-запис` | object-keep | strong candidate |
| 1150 | `слухати-мережеві-з'єднання` | `мережа-слух` | object-root | test server/listen meaning |
| 1151 | `прочитати-файл` | `файл-прочит` | object-root | strong candidate |
| 1152 | `записати-файл` | `файл-запис` | object-keep | strong candidate |

This set intentionally contains more than the required 30 examples and includes both likely successes and risky candidates. The blind test is supposed to reject some of them.

## Negative examples

The following patterns must not be promoted without extraordinary evidence:

| Full | Bad compact | Why |
|---|---|---|
| `помножити` | `пмж` | consonant code, not a readable root |
| `обчислити` | `бчс` | vowel-stripped code |
| `перший-символ-тексту` | `пст` | initials require memorization |
| `встановити-елемент-вектора!` | `вев!` | initials destroy lexical roots |
| `монотонний-час-у-наносекундах` | `мнс` | stacked abbreviation is opaque |
| `поточний-всесвітній-координований-час` | `всч` | acronym requires prior terminology knowledge |
| `мілісекунди-із-наносекунд` | `мс-нс` | unit initials are not self-decoding |
| `прочитати-з-мережевого-з'єднання` | `мрж-чт` | multiple clipped/vowel-stripped segments |
| `елемент-вектора` and `елемент-вектора-за-індексом` | same `вектор-елем` | cross-ID collision |
| `елемент-списку-за-індексом` | `за-номером` as automatic compact derivation | intuitive synonym, but not root-preserving/reverse-decodable from full wording |

## Machine-checkable invariants

Candidate tooling should enforce:

1. every candidate is keyed by a numeric semantic ID that exists in the registry;
2. candidate contains no ASCII Latin letters;
3. candidate is not longer than the current `ukr` spelling unless explicitly `keep`;
4. candidate spellings are unique across semantic IDs;
5. if full spelling ends in `?`, compact must end in `?`;
6. if full spelling ends in `!`, compact must end in `!`;
7. each changed candidate declares a rule class (`root`, `structural`, `object-root`, etc.);
8. candidates remain non-authoritative until blind-test status is `clear`;
9. promotion preserves the previous stable `uk` spelling as compatibility-only;
10. generated tables never infer a compact name on their own.

The machine cannot prove “intuitive”; it can only reject obvious policy violations. Human blind-decoding evidence owns the final promotion decision.

## Blind-decoding protocol integration

The candidate fixture should be usable to generate a questionnaire that shows only the compact spelling. The tool must not expose semantic ID or full `ukr` wording in the participant view.

For each candidate collect only:

```text
compact
full-form guess
semantic-action guess
confidence
```

Aggregate result per semantic ID:

```text
clear | ambiguous | revise | reject
```

No personal participant data belongs in the repository.

## Acceptance criteria

This design is ready for implementation when:

- `uk` and `ukr` roles above are accepted;
- compact candidates are evidence, not authority;
- candidate generation/audit uses semantic IDs to join to registry truth;
- at least 30 candidates cover the classes required by #89;
- no production `uk` rename happens before at least one blind-test pilot;
- after promotion, old `uk` source remains executable through compatibility-only aliases.
