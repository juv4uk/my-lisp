; language-contract.my — the machine-readable semantic-contract version
; for my-lisp, covering exactly Level 1 (CORE SEMANTICS: seven primitives,
; lambda, truth/NIL, symbols, pairs) and Level 2 (LANGUAGE CONTRACT:
; exactness, def/defmacro, errors, read/eval) from
; docs/language-core-axioms.md — deliberately NOT Level 3 (ECOSYSTEM
; CONFORMANCE: core.my/unify.my/reason.my/knowledge.my, literate markdown,
; CLIPS), which changes independently and far more often.
;
; Proposed 2026-08-10 to synchronize my-lisp, fpga-lisp, and cml (a new
; AOT compiler from my-lisp source to fpga-lisp's ISA) around a real
; question — "which semantic contract do you implement" — instead of a
; commit SHA or a shared release-version number. Tying all three to one
; version (e.g. "everyone is 0.15.0") was explicitly considered and
; rejected: it creates a false dependency where my-lisp could ship ten
; purely additive/library/dedup releases (exactly what happened this
; session: quotient/mod, nth/assoc/member?, cddr/cadddr/fifth,
; write-file-bytes) while the actual Level 1/2 semantic contract never
; moved — and vice versa, fpga-lisp's own ISA can change (a new opcode
; encoding, a register convention change) without my-lisp's semantics
; changing at all. Compatibility is a PAIR of versions
; (language-contract, ISA-version), not one shared number.
;
; major: bumped on a breaking semantic change — an existing, previously
; well-defined program could now observe different behavior. Examples
; from this project's own history, had this file existed then: the
; exactness model (numeric Number(f64) -> Number(f64, Exactness), item
; 10) would have been a major bump — same syntax, different observable
; truth about (eq 3 3.0).
;
; minor: bumped on an additive, backward-compatible semantic change —
; existing programs keep behaving identically, but the contract now
; recognizes something new. Examples from this project's own history:
; \r joining \n/\t as a real string escape (2026-08-10) — new *reader*
; semantics (Level 2), not something a well-formed pre-existing program
; could have depended on the old (broken) behavior for on purpose.
;
; What does NOT bump this file: adding a lib/core.my function
; (quotient, nth, assoc, ...), fixing a lib/*.my bug (the quotient/mod
; stack-overflow and divide-by-zero fixes, same day), fixing CI, adding
; a Rust primitive whose capability boundary was already Level-2-covered
; (write-file-bytes is the same "file I/O" capability class read-file/
; write-file already established) — all Level 3 or implementation
; detail, not a change to what "my-lisp" as a language promises.
;
; Starts at 0.1 here, 2026-08-10 -- not retroactively numbered for every
; semantic change already made before this file existed (exactness,
; variadic lambda, Lisp-1/Lisp-2 resolution, etc.) -- this versioning
; axis is new today, not a historical ledger.
;
; language-contract.my — машинно-читана версія семантичного контракту
; my-lisp, що покриває рівно Рівень 1 (СЕМАНТИКА ЯДРА) і Рівень 2
; (КОНТРАКТ МОВИ) з docs/language-core-axioms.md — свідомо НЕ Рівень 3
; (ЕКОСИСТЕМНА КОНФОРМНІСТЬ), який змінюється незалежно й значно
; частіше.
;
; Запропоновано 2026-08-10 для синхронізації my-lisp, fpga-lisp і cml
; (новий AOT-компілятор з my-lisp у ISA fpga-lisp) навколо реального
; питання — "який семантичний контракт ти реалізуєш" — замість SHA
; коміту чи спільного номера релізу. Прив'язка всіх трьох до одного
; номера версії була явно розглянута й відхилена: вона створює фальшиву
; залежність, де my-lisp міг би випустити десять суто адитивних/
; бібліотечних/dedup-релізів (точно те, що сталось цієї сесії:
; quotient/mod, nth/assoc/member?, cddr/cadddr/fifth,
; write-file-bytes), поки сам семантичний контракт Рівня 1/2 узагалі не
; зрушив — і навпаки. Сумісність — це ПАРА версій (language-contract,
; ISA-version), не одне спільне число.
;
; major: піднімається на breaking-зміні семантики. minor: на адитивній,
; зворотносумісній зміні семантики. Що НЕ піднімає цей файл: додавання
; функції в lib/core.my, фікс бага в lib/*.my, фікс CI, новий
; Rust-примітив у вже покритому Рівнем 2 класі можливостей.
((major . 1) (minor . 0)
 (note . "bare integer literals are exact at arbitrary precision; breaking correction from contract 0.1, 2026-08-11 · голі цілі літерали точні з довільною точністю; несумісне виправлення контракту 0.1")
 (covers . (G1 G2 G3 G4 G5 G6 G7 G8 S1 S2 S3)))
