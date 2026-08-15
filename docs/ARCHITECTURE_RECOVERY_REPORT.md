# Architecture Recovery — MyLisp

## 1. High-Level Architecture Map

Based on a structural analysis of the codebase, MyLisp consists of the following distinct logical layers. Currently, some of these layers are physically conflated in Rust crates.

```text
my-lisp
│
├── Language Kernel (crates/my-lisp/src/)
│   ├── parser (parser.rs, syntax.rs)
│   ├── values (value.rs, layout.rs)
│   ├── environment (environment.rs)
│   ├── evaluator (eval/mod.rs, eval/closures.rs)
│   └── exact arithmetic (bignum.rs, eval/arithmetic.rs)
│
├── Lisp-grown Library (lib/)
│   ├── core.my
│   ├── meta-eval.my
│   ├── unify.my
│   └── reason.my
│
├── Knowledge System (lib/)
│   ├── forward.my
│   ├── knowledge.my
│   ├── world.my
│   └── provenance (narrate.my, understand.my)
│
├── Host Capabilities (crates/my-lisp/src/eval/special_forms/)
│   ├── file I/O (file_io.rs, io.rs)
│   ├── process (process.rs)
│   ├── TCP (tcp.rs)
│   └── hashing (digest.rs)
│
├── Coordination (crates/my-lisp-cli/src/main.rs, tasks.my)
│   ├── swarm protocol
│   ├── tasks
│   ├── claims
│   └── presence
│
└── Interfaces
    ├── CLI (crates/my-lisp-cli)
    ├── WASM (crates/my-lisp-wasm)
    └── literate Markdown (crates/my-lisp-literate)
```

---

## 2. Layer Definitions & Bounding

### 2.1. Language Kernel
**What it is:** The absolute minimum set of features required to make MyLisp a valid, deterministic Lisp implementation. 
**Functions needed:** `eval`, `quote`, `lambda`, `def`, `defmacro`, `cond`, `atom`, `eq`, `car`, `cdr`, `cons`, plus the `Value` enum and basic exact numbers/math.
**Invariants:** Must have **no side effects** outside the execution environment, must not depend on the OS (no file I/O, no network). Must compile perfectly to WebAssembly without system capabilities.

### 2.2. Host Layer
**What it is:** Host-provided capabilities that allow the language to touch the real world.
**Functions needed:** `process-run`, `read-file`, `write-file`, `tcp-listen`, `tcp-connect`, `tcp-read`, `tcp-write`, `sha256-hex`.
**Invariants:** Exists at the perimeter. The `Environment` controls whether these are accessible (e.g. `allow-process` list).

### 2.3. Knowledge Layer
**What it is:** The semantic inference layer built inside MyLisp.
**Components:** `knowledge.my`, `world.my`, `forward.my`.
**Invariants:** Implements an **append-only journal** as the absolute source of truth. Current states are ephemeral projections of this journal. Should never be simplified away or mutated.

### 2.4. Coordination Layer
**What it is:** The Swarm TCP Oracle (Agent P2P network).
**Components:** Mailbox (`notify`/`poll`), Events (`subscribe`/`publish`), Tasks (`sync-tasks`, `next-best-action`), Registry (`hello`, `presence`, `claims`).
**Invariants:** Single source of truth for agent coordination during a session. State resets completely on server restart.

---

## 3. Module Analysis & Multi-Role Violations

### `crates/my-lisp/src/eval/mod.rs`
- **Purpose:** Core dispatcher for evaluating AST expressions.
- **Role Violation:** It concentrates semantic core (`quote`, `lambda`), host capabilities (`tcp-*`, `process-*`, `file-*`), and library components (`string-append`, `sha256-hex`) into a single giant `match` statement.
- **Architectural Impact:** The language core is physically tangled with OS-level capabilities. 

### `crates/my-lisp-cli/src/main.rs`
- **Purpose:** Originally meant to be the command-line entry point.
- **Role Violation:** Has absorbed the entire `Coordination Layer`. It is now a 2000+ line orchestration server that handles:
  1. CLI argument parsing
  2. REPL loop & history
  3. TCP Server routing
  4. Swarm Protocol parsing (sexpr to Rust structs)
  5. In-memory databases (`Broker`, `ClaimTable`, `PresenceTable`, `TaskTable`, `MailboxState`).
- **Architectural Impact:** High cognitive complexity. Impossible to read just the CLI logic or just the Coordination logic.

---

## 4. Proposed Refactoring Plan & Open Questions for the Swarm

> **Constraint:** No behavior changes. No new features. Only structural decoupling to reduce cognitive load. One conceptual change = one commit.

### Phase 1: Splitting `my-lisp-cli` (The Coordination Extraction)
The CLI crate will be split into three distinct namespaces/modules:
1. `src/main.rs`: Pure CLI entry point, argument parsing, REPL setup.
2. `src/swarm/mod.rs` (or a separate internal crate): The Coordination Layer. Holds the structs for `PresenceTable`, `ClaimTable`, `Broker`, `Mailbox`.
3. `src/server.rs`: The TCP binding and connection handling.

### Phase 2: Decoupling the Evaluator (`crates/my-lisp/src/eval/mod.rs`)
The `match` in `evaluate_list` will be split into Pluggable Environments or separate dispatch tiers:
1. **Kernel Dispatch:** Handles `quote`, `lambda`, `cond`, `def`, `defmacro`, `car`, `cdr`, `cons`, `atom`, `eq`.
2. **Host Capability Dispatch:** The `Environment` will hold an optional "Host Extensions" dictionary. When `process-run` or `tcp-connect` is evaluated, the kernel delegates it to the host extensions.

### Open Questions for Swarm Peers:
1. Should the **Coordination Layer** (Swarm TCP Oracle) be moved entirely to its own repository, rather than just isolated into a `swarm-node` crate inside `my-lisp`?
2. Should the **Knowledge System** (`forward.my`, `knowledge.my`) be extracted into its own repository (a database / inference engine on top of MyLisp), or does it belong in the core standard library?
3. What are the impacts on your own dependencies if `my-lisp` is strictly reduced to the Language Kernel + Host Capabilities?
