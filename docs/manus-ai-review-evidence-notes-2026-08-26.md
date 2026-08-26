# my-lisp — проміжні evidence notes

**Зріз:** `main` @ `a662dc76c99312c218d5a81047bd2a86ae316886`, клон 2026-08-26.

Супровідні докази до [`manus-ai-review-2026-08-26.md`](manus-ai-review-2026-08-26.md) — той самий автор (Manus AI), той самий commit.

## Підтверджений inventory

| Параметр | Значення |
| --- | --- |
| Workspace crates | 8: core, CLI, host, WASM, literate, LSP, semantic, swarm-node |
| Rust source files | 84 |
| `.my` files | 52 |
| Markdown docs | 106 |
| Файли, що збігаються з test path | 54 |
| Core dependencies | 0 declared dependencies у `crates/my-lisp` |

## Підтверджені сильні сторони

1. `crates/my-lisp` дійсно capability-free: filesystem/process/TCP винесені в `my-lisp-host`; core не має прямого OS access.
2. Parser має UTF-8 spans, explicit dotted pairs, відсутність quote-sugar для `'`, arbitrary-precision exact literal path, named failure для valid decimal понад parser resource limit і reader depth cap.
3. Evaluator використовує trampoline для tail calls. Batch-1 primitives (`car`, `cdr`, `cons`, `eq`, `atom`, arithmetic) зареєстровані як first-class `Value::Builtin`, тоді як syntax forms лишаються explicit dispatcher.
4. `Rational` тримає normalized arbitrary-precision numerator/denominator; output розмежовує read-back-safe `print` та human-facing `princ`; `Value::Drop` має iterative handling deep Pair values.
5. `lib/world.my` є data-first immutable history protocol: structural-sharing journals, explicit parent lineage, pure `reason-in-world`/`forward-in-world`, atomic accept-or-original-world ingest і content-address key через canonical `write-to-string`.
6. `lib/forward.my` реалізує forward chaining, JTMS multi-justification layer, cascade retraction і окремі single-condition/legacy layers; architecture consciously keeps layers additive rather than silently replacing them.

## Підтверджені current concerns

1. **Current GitHub CI is red on HEAD.** `cargo clippy --workspace` success; `cargo test --workspace` failure: `language_items::tests::every_root_builtin_is_discoverable_exactly_once`, because new runtime builtin `*argv*` has fallback metadata `(builtin ...)` rather than explicit signature/documentation.
2. `*argv*` also has a semantic split: `builtins.rs` installs it as zero-argument callable returning a vector made from its own arguments (therefore empty), while CLI overwrites the same binding with a list of script arguments before file execution. Contract should choose one model; CLI variable semantics are currently clearer.
3. `builtins.rs` contains a duplicate `string-slice` registration; the later registration overrides the earlier implementation. This is no user-visible second definition, but clear cleanup work.
4. `Environment` names a lifecycle risk: very deep lexical frame chains can still use recursive Rust `Drop`, unlike the explicit deep-pair drop mitigation.
5. Native CLI calls `my_lisp_host::install()` by default, activating unrestricted filesystem/TCP capabilities for locally run code; only `process-run` has per-session allowlist. This is an explicit trusted-local surface, not a sandbox.
6. `docs/language-core.md` has visible drift: it labels several first-class builtins as "Built-in Forms", and its Ukrainian bignum paragraph still describes a planned C core while current README says that line was explicitly dropped. README English also duplicates LSP and swarm-node entries.

## Семантичні та системні шари

| Шар | Підтверджений стан | Межа / caveat |
| --- | --- | --- |
| Closures/macros | Lexical capture, exact/variadic arity, tail-position transfer into evaluator loop; macro arguments are unevaluated data. | `value_to_expr` consciously rejects Builtin, Vector, Closure/Macro і TCP resource values as code. |
| Worlds | `lib/world.my` is executable immutable data protocol; `crates/my-lisp/tests/world.rs` declares 54 active tests over history, rollback, bridge and content identity. | `world?` is intentionally shallow (head marker), not a complete structural validator for arbitrary malformed values. |
| Forward/JTMS | `lib/forward.my` has ordinary facts, single-justification TMS and multi-justification JTMS as explicit additive layers; final JTMS loop compares full structures, not only count. | The library preserves legacy layers rather than pretending all callers now use JTMS. |
| Content identity | Canonical read-back-safe text key precedes hashing; content store is an immutable persistent-map wrapper. | Address is variable-length semantic key, not integrity/authentication hash. |
| FASL | Versioned AST cache with embedded source SHA-256 and caller-side stale fallback to text parsing; CLI compiles `core.my` text+FASL into binary. | Snapshot spans are zero; it is a cache, not a source-of-truth artifact. |
| WASM | Persistent browser session with `core.my` preloaded, explicit reset, pure/literate modes and parser diagnostics. | No host capabilities are installed, so browser execution is portable core, not native CLI parity. |
| Semantic crate | Sanskrit/SLP1/IAST/Devanagari, atom registry and kāraka structures exist. | Explicitly experimental and not wired into reader/evaluator. |
| Swarm node | Separate coordination plane with journal replay, Lamport clocks, anti-entropy, task claims/fencing, gossip and liveness controls. | No protocol-level cryptographic identity; default localhost is intentional, cross-machine bind is explicit. Cross-ecosystem subscribe/semantic-analysis gaps are recorded as open. |

## Test topology

Static inventory finds **624 declared `#[test]` attributes** under `crates/`, of which **8 are ignored**. The current CI failure occurs in the core library test binary after `63 passed; 1 failed`, so this run must not be represented as an end-to-end pass of every integration suite. The ignored tests include 6 CLI TCP S-expression protocol cases whose root cause is documented as an unresolved test-binary-specific `ConnectionRefused` issue, and 2 proposal-stage first-class-builtin cases.

The failing commit chain is precise: `80b4436` introduced the `*argv*` root builtin as a placeholder "needs env wiring"; `my-lisp-cli` already provides the real user-facing contract by binding `*argv*` as a list before a file runs. `language_items` correctly detects that the new ordinary builtin lacks metadata, which blocks current `cargo test --workspace`. The smallest repair is therefore to **remove the placeholder root builtin**, keep CLI injection as the one authority, and retain the CLI E2E contract; no new language primitive is required.

## GC and memory

`docs/gc-m0-design.md` is explicitly a **PROPOSED DESIGN**, not an implemented collector. Current Rust runtime lifetime is owned by `Rc`/`RefCell` plus an iterative `Drop` mitigation for deep Pair graphs. M0 proposes explicit `ObjectId+generation`, non-moving mark-and-sweep, explicit roots, stress mode, independent reachability oracle and metamorphic semantic equality. `memory-layout-contract.my` is likewise future cross-ecosystem NaN-boxing specification; current `Value` remains a Rust enum. Its `#xfff` marker is reader syntax that current parser treats as a symbol rather than a hex numeric literal, so it is not directly machine-readable as a numeric contract without a decoder/notation rule.

## Test environment note

The restored sandbox has no `cargo` executable, so local test execution cannot be performed here. This is an environment limitation, not a project test result. GitHub Actions log is used as external CI evidence.

## Evidence paths

Referenced by the review's author (Manus AI) as their own local working files during the audit — these paths are on the reviewer's own machine (`/home/ubuntu/...`), not this ecosystem's; listed here only for provenance/traceability of the review itself, not as files that exist in this repo or session.

- `/home/ubuntu/my-lisp-initial-inventory.txt`
- `/home/ubuntu/my-lisp-structure-map.txt`
- `/home/ubuntu/my-lisp-crate-map.txt`
- `/home/ubuntu/my-lisp-head-ci-details.json`
- `/home/ubuntu/my-lisp-head-failed-ci-log.txt`
- `/home/ubuntu/my-lisp-head-failed-ci-error-extract.txt`
- `/home/ubuntu/my-lisp-ci-execution-scope.txt`
- `/home/ubuntu/my-lisp-argv-feature-diff.txt`
- `/home/ubuntu/my-lisp-test-inventory.txt`
- `/home/ubuntu/my-lisp-audit-integrity.txt`
