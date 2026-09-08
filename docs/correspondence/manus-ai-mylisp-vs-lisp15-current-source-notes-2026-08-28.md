# Нотатки поточного коду my-lisp для порівняння з Lisp 1.5

**Статус:** супровідний доказовий документ до
`manus-ai-mylisp-vs-lisp15-comparison-2026-08-28.md` (Manus AI,
2026-08-28) — детальна таблиця відповідності поточного my-lisp коду
концептам paper Маккарті, друга половина evidence-notes (перша —
`manus-ai-mylisp-vs-lisp15-primary-source-notes-2026-08-28.md`, боку
McCarthy paper).

**Snapshot:** `ec1e149c54df8cdaacf8c1406f2f2ab79c7c79b1`, queried
2026-08-28.

| Historical concept | Current my-lisp evidence | Accurate comparison status |
|---|---|---|
| Machine-independent recursive core | `crates/my-lisp` declares capability-free core; filesystem/process/sockets live in `my-lisp-host` and embedding decides installation. | Strong conceptual continuation, with explicit modern trust boundary. |
| `eval` and application | `eval/mod.rs` parses/evaluates expressions; dispatches fixed special forms; then ordinary builtin/closure/macro invocation. Trampoline represents tail calls as data instead of recursive Rust calls. | Same central evaluator idea, modern implementation and a larger semantic surface. |
| Object-language universal evaluator | `lib/meta-eval.my` defines an intentionally bounded `my-eval` over ordinary association-list environments and dispatches the five primitive operations without reimplementing them. | The closest direct descendant of the paper's `apply`/`eval`; explicitly a demonstration, not the mandatory runtime evaluator or a claim of full language completeness. |
| `quote`, `cond`, `lambda`, recursive named definitions | Kernel has literal `quote`, `cond`, `lambda`, `def`, `defmacro`; `def` inserts into captured shared lexical frame so a closure can recurse. | Same family of mechanisms, not byte-for-byte Lisp 1.5's `LABEL`/alist semantics. |
| Atoms/pairs and list notation | Parser has `ExprKind::Pair`; dotted list folds to nested pairs. Runtime `Value::Pair`; `cons`, `car`, `cdr`, `atom` and atom-only `eq`. | Direct semantic inheritance, now protected by spans/depth limit/named errors. |
| Symbolic functions as symbolic data | `quote`, `read`, `eval`, macro expansion and `value_to_expr` model code/data transition. Closures/resources/builtins deliberately refuse to masquerade as syntax. | Homoiconicity preserved for syntax; full runtime reflection deliberately not claimed. |
| Environment / association pairs | `Environment` uses chained lexical `Frame { HashMap, parent }`; closures capture their defining environment; `get` walks outward. | Intentional advance over paper's association-list context and early dynamic-stack behavior. |
| Truth / NIL | `Value::Nil` and `Bool(false)` are falsey; empty list prints as `()` and is canonical false at syntax level. | Directly aligned with 1960 NIL tradition, while Rust keeps an internal false representation for values. |
| Memory representation | Current Rust values use `Rc`, `RefCell`, `Arc`, `HashMap`; pairs have iterative custom Drop to avoid deep structural drops. | Fundamentally different from IBM 704 address/decrement list cells; representation intentionally non-contractual. |
| Reclamation | Current runtime relies on Rust ownership/refcount lifecycle; M0 mark-sweep heap is documented design, not current implementation. Opt-in cons/numeric caps fail named. | Philosophically resumes explicit GC questions but does not yet implement 1960-style tracing collector. |
| More than 1960 formal core | Exact arbitrary-precision rationals, Unicode strings/spans, FASL SHA-256 source binding, structured errors, TCP opaque resources, numeric buffers, macro support, host-gated capabilities and conformance tiers. | Modern extensions that can coexist with minimal S-expression core; should not be attributed to the 1960 paper. |

## Current historical self-description worth preserving

`docs/language-core-axioms.md` calls its status "draft, not yet
ratified" and separates three levels: core semantics, language
contract and ecosystem conformance. It explicitly says its own target
is not surface imitation of a single dialect, and treats the
symbolic-reasoning layer as the project's motivating layer, not a
core-language fact.

The document also gives useful current boundaries: exact decimals are
rational by default; resource limits must produce named errors; full
multi-substrate conformance is supported only for a subset; lexical
scope is implemented; controlled dynamic binding is absent and
consciously left as an open tool-design question, not retroactively
treated as a defect in early Lisp.

## Important non-equivalences

- Paper `LABEL` is an explicit S-expression representation of
  recursion. Current `my-lisp` has no `label` operator: host-level
  `def` makes a recursive closure visible by inserting it into a
  shared lexical frame; `lib/meta-eval.my` candidly does **not**
  support recursive top-level `def` because its immutable alist
  environment captures the pre-binding environment.

- Current implementation registers `atom`, `eq`, `car`, `cdr`, and
  `cons` as first-class root `Builtin` values. Therefore they are
  callable values and can be shadowed as ordinary bindings; an older
  statement in `language-core-axioms.md` that all seven primitives are
  literal unshadowable dispatch is not fully synchronized with the
  current `eval/builtins.rs` contract.

- Current parser deliberately treats apostrophe as an identifier
  character, not quote sugar, preserving Ukrainian identifiers such as
  `об'єкт`; explicit `(quote ...)` remains the portable quotation form.

## Static cycle-safety witness: modern exception to the 1960 constraint

The paper explicitly forbids cyclic list structure, partly because
printing and other operations become difficult. Current `Pair` values
are immutable from the language, so normal cons-lists do not gain a
cycle constructor. However, current `Vector` is mutable
(`Rc<RefCell<Vec<Value>>>`) and `vector-set!` accepts an arbitrary
`Value` without rejecting self-reference, while `render` and
`Value::PartialEq` recursively descend vectors with no visited-identity
set.

The following is therefore a **static risk witness**, not an executed
test result:

```lisp
(def v (vector (quote ())))
(vector-set! v 0 v)
(write-to-string v)  ; recursive renderer has no cycle guard
;; and (eq v v) reaches structural vector equality with the same issue
```

If this reading is confirmed by a future local regression test, it is
a genuine modern analogue of the problem the 1960 system avoided by
disallowing circular structure. The minimal safe choice need not be a
new cycle-capable data model: either reject indirect/self-containing
`vector-set!` operations until printer/equality semantics are
designed, or add deterministic cycle detection and a named
outcome/rendering convention. This point belongs to future validation;
it does **not** invalidate current acyclic core conformance.

**Не виконано жодною сесією досі** — ні звітом Manus AI, ні цією
сесією (wsl-nidana-1): статичний witness, не запущений тест. Хто б не
брав цю задачу далі — перший крок дешевий: написати саме цей
regression test і подивитись, що станеться насправді, перш ніж
вирішувати дизайн cycle-safety.
