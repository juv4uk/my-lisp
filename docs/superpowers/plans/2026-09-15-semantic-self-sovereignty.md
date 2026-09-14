# Semantic Self-Sovereignty Implementation Plan

## Українське резюме

Цей план реалізує лише документаційний шар схваленого дизайну: design note, trust-root/performance boundary, Error Observation Identity v1, окремий future gate #145 і розширення research queue #132. Він **не змінює runtime, evaluator, compiler, semantic IDs чи witness truth** і не торкається активної гілки PR #144.

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Preserve the approved semantic-self-sovereignty design as durable design capital while tightening existing authority boundaries without changing runtime semantics.

**Architecture:** The design note is the umbrella statement. Existing compiler and error boundary documents receive only clarifications that are already implied by their current decisions: today’s named trust root, one-way performance policy, and Error Observation Identity v1. Future falsification and prior-art work remain separate issues rather than becoming implementation scope in this PR.

**Tech Stack:** Markdown design/ADR documents, GitHub issues, existing CI/docs checks.

**Spec:** `docs/research/2026-09-15-semantic-self-sovereignty.md`

## Global Constraints

- Do not modify `lib/`, `crates/`, evaluators, compiler code, semantic IDs, or witness expected values.
- Do not modify PR #144 or its branch.
- Do not expand ADR-011’s current error contract; name the existing category-only cross-runtime identity as v1.
- Performance policy must remain downstream of semantic admission with no backward authority edge.
- HOST-SURGERY-1 is future work blocked by #116 and must pin a corpus/witness digest before surgery.
- Prior-art additions are research candidates only; no similarity or uniqueness verdict is allowed without Rule-15 evidence.

---

### Task 1: Preserve the design-capital note

**Files:**
- Create: `docs/research/2026-09-15-semantic-self-sovereignty.md`

**Interfaces:**
- Consumes: existing Vertical Day record, compiler authority boundary, ADR-011, issues #113–#116 and #132.
- Produces: the umbrella status vocabulary and claim boundary referenced by later document clarifications.

- [ ] **Step 1: Write the dated design note**

Include `RATIFIED`, `HYPOTHESIS`, and `ILLUSTRATION, NOT COMMITMENT`; current trust root; performance non-interference; Error Observation Identity v1; precise CML wording; HOST-SURGERY digest rule; claims made/not made.

- [ ] **Step 2: Review for overclaims**

Confirm the note does not claim certified compilation, completed #113–#116, eliminated Rust trust, complete native runtime, or prior-art uniqueness.

- [ ] **Step 3: Commit**

```bash
git add docs/research/2026-09-15-semantic-self-sovereignty.md
git commit -m "docs: define semantic self-sovereignty design capital"
```

### Task 2: Tighten the compiler authority boundary

**Files:**
- Modify: `docs/COMPILER-AUTHORITY-BOUNDARY.md`

**Interfaces:**
- Consumes: the trust-root and performance rules from the design note.
- Produces: explicit boundary language that compiler/backend consumers can cite without reading the whole research note.

- [ ] **Step 1: Add current trust-root section**

State that Lisp-owned corpus data may be normative while the native evaluator/reader/runtime remain part of today’s trusted execution substrate; independent disagreement is evidence, not something to normalize away.

- [ ] **Step 2: Add performance non-interference section**

State that admitted implementations are semantically equal only for covered witnesses; benchmark/scheduler policy may select among them but cannot change semantic IDs, expected outcomes, or admission rules by backpressure.

- [ ] **Step 3: Run docs/diff hygiene through PR CI**

Expected: existing documentation and fast policy checks remain green; no runtime test behavior changes.

- [ ] **Step 4: Commit**

```bash
git add docs/COMPILER-AUTHORITY-BOUNDARY.md
git commit -m "docs: name trust root and performance boundary"
```

### Task 3: Name Error Observation Identity v1 without expanding it

**Files:**
- Modify: `docs/adr/ADR-011-ERROR-DETAIL-CONTRACT-BOUNDARY.md`

**Interfaces:**
- Consumes: ADR-011’s accepted category-over-detail decision.
- Produces: a concise identity rule for #113–#116 and future backend conformance.

- [ ] **Step 1: Add Error Observation Identity v1 subsection**

Record `cross-runtime semantic error identity = admitted error category`; explicitly keep message, span, meta detail, and unbound-symbol payload diagnostic unless a future ADR ratifies them.

- [ ] **Step 2: Preserve the future-extension gate**

Require explicit ADR/contract decision + implementation-neutral machine-readable schema + shared executable witnesses before any detail field can become identity-relevant.

- [ ] **Step 3: Commit**

```bash
git add docs/adr/ADR-011-ERROR-DETAIL-CONTRACT-BOUNDARY.md
git commit -m "docs: name error observation identity v1"
```

### Task 4: Create deferred evidence/research gates

**Files:**
- GitHub issue: create `HOST-SURGERY-1` blocked by #116.
- GitHub issue #132: append research-queue candidates.

**Interfaces:**
- Consumes: approved future-test and prior-art requirements.
- Produces: deferred work that cannot leak into the current docs PR scope.

- [ ] **Step 1: Create HOST-SURGERY-1**

Acceptance must require a pre-surgery digest over the authoritative corpus/witness inputs, CI equality of the digest after surgery, unchanged witness definitions, mechanical-only implementation change, and green applicable semantic witnesses.

- [ ] **Step 2: Extend #132 research queue**

Add golden/reference models, Sail/RISC-V, CompCert/translation-validation literature, WASM multi-engine conformance, Java TCK/JCK, and definitional/semantics-first systems including CakeML, K Framework, and PLT Redex. Mark every entry as candidate-only with no verdict.

- [ ] **Step 3: Open the design-capital PR**

PR must be docs/governance-only and explicitly state that #144 is untouched.

- [ ] **Step 4: Verify changed-file scope and CI**

Expected changed repository files: the design note, this plan, compiler boundary, ADR-011. GitHub issue mutations are not part of the PR diff.
