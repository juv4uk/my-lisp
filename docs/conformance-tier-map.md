# conformance.json — tier and axiom map (working draft)

Companion to `docs/language-core-axioms.md`, not a replacement for `conformance.json` itself — nothing in the actual fixture file changes here. This is the "next step" pass promised in that document: every fixture, in order, tagged with its tier (1 = CORE SEMANTICS, 2 = LANGUAGE CONTRACT, 3 = ECOSYSTEM CONFORMANCE) and, where one clearly applies, the axiom (G1–G7, S1–S3) it's evidence for.

**Status: draft, not yet ratified.** Produced 2026-08-09 as a discussion basis, not a final classification.

| # | expr | Tier | Axiom | Note |
|---|---|---|---|---|
| 1 | `(quote radio)` | 1 | — | core primitive itself |
| 2 | `(atom 'radio)` | 1 | G2 | |
| 3 | `(atom '())` | 1 | G2 | NIL is also an atom |
| 4 | `(atom '(radio antenna))` | 1 | G2 | |
| 5 | `(eq 'radio 'radio)` | 1 | G1 | |
| 6 | `(eq 'radio 'antenna)` | 1 | G1 | |
| 7 | `(car '(radio antenna))` | 1 | G2 | |
| 8 | `(cdr '(radio antenna))` | 1 | G2 | |
| 9 | `(cons 'radio '(antenna))` | 1 | G2 | |
| 10 | `(cond (() 'wrong) (t 'right))` | 1 | G8 | gap closed — see below |
| 11 | `(/ 5 6 8 7)` → `5/336` | 2 | S1 | |
| 12 | `(+ (/ 1 3) (/ 1 3))` | 2 | S1 | |
| 13 | `(- 1 (/ 1 3))` | 2 | S1 | |
| 14 | `(* (/ 2 3) (/ 9 4))` | 2 | S1 | |
| 15 | `(- (/ 1 3))` | 2 | S1 | |
| 16 | `(+ (/ 1 2) 0.25)` → `0.75` | 2 | S1 | exact+inexact→inexact rule |
| 17 | `(eq 3 3)` | 1 | G1 | |
| 18 | `(eq 3 4)` | 1 | G1 | |
| 19 | `(eq "radio" "radio")` | 1 | G1 | strings as atoms |
| 20 | `(cond (() 'first) (() 'second) (t 'third))` | 1 | G8 | same gap as #10 |
| 21 | `(second '(radio antenna signal))` | 3 | G5 | `lib/core.my` |
| 22 | `(third '(radio antenna signal))` | 3 | G5 | |
| 23 | `(not '())` | 3 | G5 | |
| 24 | `(not 'radio)` | 3 | G5 | |
| 25 | `(length '(radio antenna signal))` | 3 | G5 | |
| 26 | `(length '())` | 3 | G5 | |
| 27 | `(reverse '(radio antenna signal))` | 3 | G5 | |
| 28 | `(append '(radio) '(antenna signal))` | 3 | G5 | |
| 29 | `(map (lambda (x) (+ x 1)) '(1 2 3))` | 3 | G5 | `lambda` itself is tier 1/2, `map` is tier 3 |
| 30 | `(filter (lambda (x) (eq x 2)) '(1 2 3 2))` | 3 | G5 | |
| 31 | `(reduce (lambda (acc x) (+ acc x)) 0 '(1 2 3 4))` | 3 | G5 | |
| 32 | `(< 1 2 3)` | 2 | — | chained comparison, host primitive |
| 33 | `(< 1 3 2)` | 2 | — | |
| 34 | `(> 3 2 1)` | 2 | — | |
| 35 | `(= 1 1 1)` | 2 | — | |
| 36 | `(= 1/2 0.5)` | 2 | G1, S1 | value equality across exact/inexact representation |
| 37 | `(<= 1 1 2)` | 2 | — | |
| 38 | `(>= 2 2 1)` | 2 | — | |
| 39 | `(print 42)` | 2 | — | host capability (session transcript), not pure semantics |
| 40 | `(read "(+ 1 2)")` | 2 | G3 | |
| 41 | `(eval (read "(+ 1 2)"))` | 2 | G3 | |
| 42 | `(car 5)` error `Type` | 1 | S2 | |
| 43 | `(car '())` error `Type` | 1 | S2 | |
| 44 | `(eq '(1) '(2))` error `Type` | 1 | S2 | `eq` only accepts atoms |
| 45 | `(undefined-symbol)` error `UnknownSymbol` | 1/2 | S2 | |
| 46 | `(lambda (x))` error `Arity` | 2 | S2 | |
| 47 | `(quote a b)` error `Arity` | 1 | S2 | |
| 48 | `(cons 1)` error `Arity` | 1 | S2 | |
| 49 | `(/ 1 0)` error `InvalidForm` | 2 | S2 | |
| 50 | `(let ((x 1) (y 2)) (+ x y))` | 3 | G5 | `let` is a macro in `lib/core.my` |
| 51 | `(let* ((x 1) (y (+ x 1))) y)` | 3 | G5 | |
| 52 | `(equal? '(1 (2 3) 4) '(1 (2 3) 4))` | 3 | G5 | |
| 53 | `(equal? '(1 2) '(1 2 3))` | 3 | G5 | |
| 54 | `(eq (lambda (x) x) (lambda (x) x))` | 2 | G1 | closure identity, not structural equality |
| 55 | `(unify 'a 'a '())` | 3 | — | principle 3 evidence, not a G/S axiom |
| 56 | `(unify '(var x) 'a '())` | 3 | — | |
| 57 | `(unify '(var x) '(var x) '())` | 3 | — | |
| 58 | `(unify '(1 (var x) 3) '(1 2 3) '())` | 3 | — | |
| 59 | `(unify '(1 (var x) 3) '(1 2 4) '())` | 3 | — | |
| 60 | `reason` — simple fact proof | 3 | — | principle 3 |
| 61 | `reason` — with `logic-var` | 3 | — | principle 3 |
| 62 | `reason` — bird/penguin proof tree | 3 | — | principle 3, the deepest fixture in the file |
| 63 | `(equal? '(p . 0) (cons 'p 0))` | 1 | G2 | today's dotted-pair fix; derived — `equal?` is a `lib/core.my` function, not one of the seven primitives, even though `cons` sits inside it |
| 64 | `(car '(a b . c))` | 1 | G2 | today's dotted-pair fix; constitutive — `car` invoked directly |
| 65 | `(cdr (cdr '(a b . c)))` | 1 | G2 | today's dotted-pair fix; constitutive — `cdr` invoked directly |
| 66 | `(unify '(var x) '(f (var x)) '())` | 3 | — | occurs-check, principle 3 evidence — prevents building an infinite structure |
| 67 | `(defmacro foo)` error `Arity` | 2 | S2 | `defmacro` validates arity like any other special form |
| 68 | `(defmacro 5 (x) x)` error `InvalidForm` | 2 | S2 | `defmacro` validates its name is a symbol, not just its arity |
| 69 | `(def count-down (lambda (n) (cond ((eq n 0) 'done) (t (count-down (- n 1)))))) (count-down 100000)` | 2 | S3 | 100,000-deep self-tail-call stays O(1) host-stack usage regardless of depth; previously only `stack_safety.rs` (Rust-specific), not the implementation-independent contract |
| 70 | `(map (lambda (x) (+ x 1)) '())` | 3 | G5 | empty-list edge case, previously untested |
| 71 | `(filter (lambda (x) (eq x 2)) '())` | 3 | G5 | empty-list edge case, previously untested |
| 72 | `(reduce (lambda (acc x) (+ acc x)) 0 '())` | 3 | G5 | empty-list edge case — returns initial accumulator unchanged, previously untested |

**2026-08-09, later same day:** the literate-markdown fixture (formerly #42) was removed from `conformance.json` entirely — it was the file's only non-S-expression entry and duplicated coverage already owned by `crates/my-lisp-literate/tests/literate_offsets.rs`. Rows 1-65 above were renumbered to match the resulting 65-fixture file.

**2026-08-09, still later:** three fixtures appended (append-only, at the end, per the file's own convention) closing gaps found by opinion review: `unify`'s occurs-check (previously only covered by `unify.rs`'s Rust-level unit test, not the implementation-independent contract) and two `defmacro` error paths (arity, non-symbol name) — the macro system previously had zero error coverage in `conformance.json`.

**2026-08-09, even later:** one more fixture appended closing the last gap from the same review — a 100,000-deep tail-recursive call, so stack-safety-under-recursion is now part of the implementation-independent contract too, not just `stack_safety.rs`'s Rust-only coverage. (Its note originally said "O(1) Rust stack" — corrected to "host-stack usage" after review found the contract's own explanatory text had quietly assumed a Rust reader, the one place Rust-specific wording had leaked outside an explanation into something closer to a claim.)

**2026-08-09, last pass of the day:** three empty-list edge cases appended for `map`/`filter`/`reduce` — until now these had exactly one fixture each (happy path only), a real coverage gap next to `unify`/`reason`'s ~9 fixtures including a full proof tree. Not a claim that symbolic AI is over-tested — principle 3 names it as a project goal, so deeper coverage there is a deliberate choice — but `map`/`filter`/`reduce` having zero edge cases was an accident of "it just worked when written," not a decision. File is now 72 fixtures.

## A gap found by doing this pass — and closed the same day

Fixtures #10 and #20 test `cond` selecting the first true clause and `'()` acting as false — this is exactly what the Tier 1 definition in `language-core-axioms.md` already names ("truth/NIL") as part of core semantics, but no G-axiom actually stated it. Found by walking the fixtures, not predicted in advance — exactly the kind of thing this exercise exists to surface. Closed by adding **G8**: `'()` is deliberately both the empty list and the canonical false, the same choice McCarthy made in Lisp 1.5, stated explicitly rather than left implicit. Not what Scheme later did (splitting `'()`/`#f`) — a different, equally legitimate design, but not this language's own choice, which its own tests already commit to.

## Counts

- Tier 1 (CORE SEMANTICS): 23 fixtures
- Tier 2 (LANGUAGE CONTRACT): 22 fixtures
- Tier 3 (ECOSYSTEM CONFORMANCE): 27 fixtures
- Total: 72 fixtures (literate layer removed; occurs-check, 2 defmacro error fixtures, 1 tail-recursion fixture, and 3 map/filter/reduce empty-list fixtures added, all 2026-08-09; see notes above)

(Corrected 2026-08-09 from an initial hand count of 22/15/20/1 — `my-lisp-constitution.json`'s machine-checked tier field is now the authoritative count, not this table's manual tally.)
- Fixtures with no clean G/S axiom mapping: 8 (`unify`/`reason`, evidence for principle 3, not the G/S axiom list) — #10/#20 resolved by G8, no longer unmapped

This distribution is itself informative: symbolic reasoning (`unify`/`reason`) is 8 of 66 fixtures but is exactly the part principle 3 says should never be treated as optional — worth keeping in mind if the file ever splits physically, so `symbolic.json` doesn't end up looking like an afterthought relative to `language-core.json`'s larger fixture count.
