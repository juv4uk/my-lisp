; pending-replay-2026-09-14-research-vertical.lisp
;
; Структурний task fragment для clean parser/rebuild replay у root tasks.lisp.
; НЕ є другою task authority. GitHub issues #126/#131/#132/#133 є живими
; execution records до структурного merge цього fragment у tasks.lisp.
;
; Причина окремого fragment: docs/registry-mutation-policy.md забороняє
; incremental string surgery над tasks.lisp. Replay має прочитати повний
; tasks.lisp, розпарсити S-expression, додати ці 4 записи in-memory,
; повністю перевидати registry і перевірити баланс/parse перед commit.

((tasks . (
  ("COLD-START-30M-1" . (
    (priority . 8.8)
    (capabilities . (research reproducibility documentation witness native cold-start))
    (origin . owner)
    (issue . 131)
    (depends-on . (VERTICAL-LISP-2))
    (description . "Чиста машина або VM, одна canonical README/entrypoint процедура, <=30 хвилин, один Lisp-authored semantic witness + один native vertical witness. Невдача через неповну документацію рахується defect-ом документації, а не недосвідченістю користувача. Немає hidden local binaries/cache/config як authority; exact heads/toolchain/commands/outputs/hashes записані; expected semantic truth не дублюється у shell/Rust/Markdown.")
    (acceptance . "Незалежний clean run без усних підказок проходить semantic witness і #126-derived native witness; canonical entrypoint легко знаходиться; exact executed artifact/bytes мають provenance; missing/stale dependency fail-closed з діагнозом; усі friction points повернуті у docs/setup як defects/fixes.")
    (done . nil)
  ))

  ("PRIOR-ART-REGISTRY-1" . (
    (priority . 8.4)
    (capabilities . (research rule-15 prior-art history architecture evidence))
    (origin . owner)
    (issue . 132)
    (depends-on . ())
    (description . "Rule-15 prior-art registry замість есе з пам'яті. Для Scheme-79, Forth/Chuck Moore chips, GA144, CADR/Lisp-machine FPGA reimplementations, Racket #lang, Unison, wenyan, Slang/Squeak, fexpr systems, colorForth та нових знайдених кандидатів зберігати: date checked, exact queries, primary/strong sources, source-confirmed properties, relation/overlap dimensions, missing dimensions, counterexample strength, verdict, confidence/unresolved questions. Scheme-79 описувати точно як custom VLSI Scheme processor, не FPGA; FPGA prior art перевіряти окремо.")
    (acceptance . "Є versioned evidence registry з усіма стартовими кандидатами, таблицею dimensions і явними counterexamples; кожен substantive claim має джерело; unresolved не маскується як negative result; сформульовані strongest defensible claim і weaker fallback claim.")
    (done . nil)
  ))

  ("VERTICAL-LISP-2" . (
    (priority . 9.4)
    (capabilities . (research milestone lisp data-model machine x86-64 native witness))
    (origin . owner)
    (issue . 126)
    (depends-on . ())
    (description . "Research milestone, не просто implementation task: (перше (сполучити 2 3)) -> semantic IDs 0005/0004 -> one Lisp-owned pair representation contract -> Lisp-owned x86 lowering/encoding -> semantics-blind host -> physical CPU -> 2; окремо (решта (сполучити 2 3)) -> 3 через IDs 0006/0004 і той самий contract. До GREEN не стверджувати завершений research result.")
    (acceptance . "Pair layout має одне machine-readable authority; CAR/CDR/CONS lowering живе у Lisp machine layer; allocation mechanism не стає semantic authority; host не знає meaning car/cdr/cons; interpreter/native parity 2 і 3; invalid non-pair semantics збережені; exact bytes мають provenance; focused+regression CI green; milestone входить другим witness у COLD-START-30M-1.")
    (done . nil)
  ))

  ("UNIQUE-CLAIM-DISCIPLINE-1" . (
    (priority . 7.9)
    (capabilities . (research documentation epistemic-discipline prior-art claims))
    (origin . owner)
    (issue . 133)
    (depends-on . (PRIOR-ART-REGISTRY-1))
    (description . "До завершення prior-art survey активні тексти використовують лише фальсифіковне формулювання: 'Серед перевірених нами близьких систем ми поки не знайшли системи з тією самою комбінацією authority boundaries, peer semantic surfaces і executable proof discipline.' Заборонено як факт: 'жодна існуюча система', 'перша у світі', 'унікальна серед усіх мов/машин' без evidence boundary. Historical wording не стирати, а контекстуалізувати.")
    (acceptance . "Active docs не мають universal uniqueness claims без evidence boundary; #132 є canonical prior-art evidence source; #126 research wording gated executable status; counterexample веде до update claim; після #132 strongest defensible wording переглянуто доказово.")
    (done . nil)
  ))
)))
