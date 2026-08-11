# data/

External CLIPS example source files imported for local reference, not wired into the build, tests, or `lib/clips-import.my` fixtures. Kept separate from `tests/fixtures/*.clp` (the project's actual CLIPS-import conformance fixtures, sourced from the official `smarr/CLIPS` examples repo) so provenance stays unambiguous.

## Contents

### `gitonga123-career-advisor/`
Source: https://github.com/gitonga123/CLIPS (branch `master`)
Career Advisory Expert System for Computer Science/IT students — `CareerAdvisor.CLP` (rules) + `Data.CLP` (facts).
**License: unconfirmed.** No `LICENSE` file was found in the source repository at import time (2026-08-11) — verify licensing before any redistribution or reuse beyond this local copy.

### `ariosolzq-expert-system/`
Source: https://github.com/Ariosolzq/Expert-System-CLIPS (branch `main`)
Two expert systems: `Expert System-Animal Classification.clp` (animal classification, in Chinese) and `Expert System-Barley Grain Diagnosis.clp` (barley grain disease diagnosis).
**License: MIT** (`LICENSE` copied alongside).

### `wcyn-horticulture/`
Source: https://github.com/wcyn/clips-horticulture-expert-system (branch `master`)
Horticulture disease/pest diagnosis system. `diagnosis_rules_automated.CLP` dynamically generates rules at runtime from the accompanying data files: `symptoms.txt` (symptom questions), `advice.txt` (treatment advice per diagnosis), `diagnoses.txt` (diagnosis message text) — the `.CLP` file is not self-contained without them.
**License: MIT** (`LICENSE` copied alongside).

Imported 2026-08-11.
