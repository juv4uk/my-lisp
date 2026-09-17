# Repo Tooling Inventory Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement #382 as a Lisp-owned, fail-closed inventory for immediate `scripts/*` tooling, with executable negative witnesses, Guard navigation, and CI enforcement without adding Rust.

**Architecture:** `knowledge/repo-tooling-inventory.lisp` is the single machine-readable registry. `scripts/check-repo-tooling-inventory.lisp` owns schema/policy validation and observes the real immediate `scripts/` directory through existing CLI filesystem capabilities. Existing `.github/workflows/ci.yml` runs the checker; no Rust/Python verifier and no new workflow are introduced.

**Tech Stack:** my-lisp (`.lisp`), existing `read-dir`/`read-file` host capabilities through `my-lisp-cli`, GitHub Actions YAML, Guard navigation data.

**Spec:** `docs/superpowers/specs/2026-09-17-repo-tooling-inventory-design.md`

## Global Constraints

- #299 is authoritative: **zero added Rust lines and zero new `.rs` files**.
- #76 owns Python→Lisp migration; #382 records migration state and must not create a second Python roadmap.
- The inventory is governance metadata, not semantic authority.
- The first coverage domain is every immediate entry from `read-dir("scripts")` except the literal directory entry `tests`.
- Nested `scripts/tests/*` are out of scope for this slice.
- Do not move or delete existing scripts in #382.
- Do not create a new GitHub Actions workflow; reuse `.github/workflows/ci.yml`.
- New control flow must use explicit result equality (`structural-kind`, `identity-relation`, `structural-relation`) rather than reintroducing generic `t/()` truth authority.

---

### Task 1: Add RED-first pure inventory validator and negative witnesses

**Files:**
- Create: `scripts/check-repo-tooling-inventory.lisp`

**Interfaces:**
- Consumes: inventory rows shaped as `(tool (path ...) (kind ...) ...)` and observed immediate script entry names such as `("authority-guard.lisp" "deploy.sh")`.
- Produces: `repo-tooling-verdict` returning either `(repo-tooling-ok)` or the first explicit violation record; top-level real-tree enforcement is added only after the pure witnesses pass.

- [ ] **Step 1: Create the checker with explicit-result helpers and RED self-witnesses**

Start with the pure interfaces below. Do not call `read-dir` yet.

```lisp
; #382 — Lisp-owned repository tooling inventory validator.
; Policy data belongs to knowledge/repo-tooling-inventory.lisp.

(def repo-tooling-field
  (lambda (name row)
    (let ((entry (assoc name (cdr row))))
      (cond
        ((atom entry) (structural-kind empty-list) (quote missing))
        ((atom entry) (structural-kind atom) (quote missing))
        ((atom entry) (structural-kind pair) (second entry))))))

(def repo-tooling-path
  (lambda (row) (repo-tooling-field (quote path) row)))

(def repo-tooling-path-equal?
  (lambda (left right)
    (equal? (repo-tooling-path left) (repo-tooling-path right))))

(def repo-tooling-violation
  (lambda (kind detail)
    (list (quote repo-tooling-violation) kind detail)))
```

Add a pure `repo-tooling-verdict` skeleton plus self-witness data whose expected result is an unregistered-tool violation:

```lisp
(def repo-tooling-selftest-unregistered
  (lambda ()
    (repo-tooling-verdict
      (quote
        ((tool
           (path "scripts/a.lisp")
           (kind check)
           (language lisp)
           (role sample)
           (lifecycle active)
           (callers ())
           (authority-source (issue 382))
           (migration-issue ())
           (replacement ())
           (removal-condition ())))))
      (quote ("a.lisp" "b.lisp")))))
```

The witness must expect:

```lisp
(repo-tooling-violation unregistered-tool "scripts/b.lisp")
```

Until `repo-tooling-verdict` is implemented, make the script fail intentionally by evaluating an undefined/missing validator or explicit `(car ())` after checking that the witness cannot pass.

- [ ] **Step 2: Run the focused RED command**

Run:

```sh
cargo run -p my-lisp-cli --bin my-lisp -- scripts/check-repo-tooling-inventory.lisp
```

Expected: non-zero failure attributable to the missing/incomplete validator, not parser/bootstrap failure.

- [ ] **Step 3: Implement structural recursion without generic truthiness**

Implement these pure helpers using explicit `cond` expected-result clauses:

```text
repo-tooling-valid-kind?
repo-tooling-valid-language?
repo-tooling-valid-lifecycle?
repo-tooling-row-schema-verdict
repo-tooling-find-row-by-path
repo-tooling-duplicate-path-verdict
repo-tooling-observed-coverage-verdict
repo-tooling-stale-row-verdict
repo-tooling-python-migration-verdict
repo-tooling-bootstrap-removal-verdict
repo-tooling-verdict
```

Every list recursion must distinguish `(structural-kind empty-list)` from `(structural-kind pair)`. Symbol comparison uses `eq` with `(identity-relation same|distinct)`; string/path comparison uses `equal?` with `(structural-relation same|distinct)`.

Closed vocabularies must be literal Lisp data in this checker:

```lisp
(def repo-tooling-kinds
  (quote (check generator migration benchmark deploy release helper other)))
(def repo-tooling-languages
  (quote (lisp python shell javascript powershell other)))
(def repo-tooling-lifecycles
  (quote (active transitional legacy generated-helper archive-candidate
          parity-green switched-to-lisp removable bootstrap-exception)))
```

Do not use legacy `member?` as a control predicate; implement enum membership with explicit `eq` result matching.

- [ ] **Step 4: Add all required pure negative witnesses**

The checker must run and verify these cases before touching the real filesystem:

```text
unregistered observed tool
same path registered twice
registered path absent from observed names
unknown kind
a Python active/transitional row with empty migration-issue
bootstrap-exception with empty removal-condition
```

Each witness must compare the actual verdict to one exact expected violation via `equal?` + `(structural-relation same)`. A witness mismatch must force non-zero exit with `(car ())`.

- [ ] **Step 5: Run the pure witnesses GREEN**

Run:

```sh
cargo run -p my-lisp-cli --bin my-lisp -- scripts/check-repo-tooling-inventory.lisp
```

At this stage the script may finish with a named placeholder result such as `(repo-tooling-selftests-ok)` after all pure witnesses pass; it must not yet claim the real repository inventory is complete.

- [ ] **Step 6: Commit the validator/witness slice**

```sh
git add scripts/check-repo-tooling-inventory.lisp
git commit -m "test(tooling): add Lisp-owned inventory validator witnesses"
```

---

### Task 2: Add the complete current top-level scripts inventory and real-tree enforcement

**Files:**
- Create: `knowledge/repo-tooling-inventory.lisp`
- Modify: `scripts/check-repo-tooling-inventory.lisp`

**Interfaces:**
- Consumes: exact current immediate `scripts/` entries plus the registry forms read by `read-all`.
- Produces: real repository verdict `(repo-tooling-ok)` on exact coverage; any new unregistered immediate entry or missing registered immediate path fails closed.

- [ ] **Step 1: Create the machine-readable registry header**

The file begins with plain data, not evaluated definitions:

```lisp
(about
  (schema repo-tooling-inventory/1)
  (issue 382)
  (scope immediate-scripts-entries-except-tests)
  (authority governance-metadata-not-language-semantics)
  (python-migration-authority 76)
  (rust-retirement-valve 299))
```

Then add one `(tool ...)` row for every current immediate file under `scripts/`, plus the new checker itself. `scripts/tests` is deliberately excluded by the scope rule rather than represented as a tool.

- [ ] **Step 2: Populate the exact current path set**

Inventory these existing 41 files and the new checker (42 total rows after Task 1):

```text
authority-guard-enforce.lisp
authority-guard.lisp
benchmark-semantic-registry-read.lisp
benchmark.mjs
build-constitution.lisp
build-dependency-classification.lisp
build-inventory.lisp
check-bilingual-docs
check-meta-eval-evidence.py
check-multilingual-parity.py
check_semantic_registry.py
check_surface_coverage.py
check_trilingual_surface.py
compare-oracle-time.py
deploy.sh
fixtures-for-tier.lisp
gen-docs-index.sh
gen-functions.lisp
generate-function-table.lisp
generate-meta-eval-evidence.py
generate-meta-semantic-registry.py
generate-uk-surface-audit.lisp
generate-ukrainian-api.py
install-hooks.sh
machine-authority-guard.lisp
make-portable-web.mjs
oracle-batch.lisp
program-symbol-table.lisp
public_api_inventory.py
release.lisp
rust-one-way-valve.sh
semantic-ownership.py
start-local-oracle-release.sh
stop-local-oracle-release.sh
swarm-node-portproxy.ps1
symbol-table.lisp
test-current-semantic-slice.sh
test-rust-one-way-valve.sh
translate-program.py
uk-latynka.py
перевірити-український-профіль.lisp
check-repo-tooling-inventory.lisp
```

Use language classification by actual implementation:

```text
.lisp -> lisp
.py -> python
.sh and check-bilingual-docs shebang -> shell
.mjs -> javascript
.ps1 -> powershell
```

All repo-owned Python rows are `transitional`. Use `migration-issue 317` for `generate-meta-semantic-registry.py`, `351` for `generate-meta-eval-evidence.py`, and parent `76` for every other active Python tool until a narrower child exists. This records existing migration ownership without inventing new tasks.

For fields not proven by repository evidence, use `unknown` or `()`; do not invent caller or authority claims. Every row still contains every required field.

- [ ] **Step 3: Make the top-level runner observe the real directory**

At the bottom of `scripts/check-repo-tooling-inventory.lisp`, add:

```lisp
(def repo-tooling-forms
  (read-all (read-file "knowledge/repo-tooling-inventory.lisp")))

(def repo-tooling-rows
  (repo-tooling-only-tool-forms repo-tooling-forms))

(def repo-tooling-observed
  (repo-tooling-without-tests (read-dir "scripts")))

(def repo-tooling-real-verdict
  (repo-tooling-verdict repo-tooling-rows repo-tooling-observed))
```

`repo-tooling-only-tool-forms` must retain only forms whose head is `tool` by explicit `eq` result matching. `repo-tooling-without-tests` removes only the exact string `"tests"` by explicit structural equality.

If `repo-tooling-real-verdict` is `(repo-tooling-ok)`, return it. Otherwise `print` the violation and force non-zero exit using the existing Lisp fail-closed idiom `(car ())`.

- [ ] **Step 4: Run the real inventory checker GREEN**

Run:

```sh
cargo run -p my-lisp-cli --bin my-lisp -- scripts/check-repo-tooling-inventory.lisp
```

Expected final value/output includes:

```lisp
(repo-tooling-ok)
```

and exits 0.

- [ ] **Step 5: Prove fail-closed behavior without leaving mutations**

Temporarily copy the inventory to `/tmp`, remove the checker row or add a temporary unregistered file under `scripts/`, run the checker and observe non-zero, then restore/remove the temporary mutation before commit. The committed tree must remain unchanged by this proof.

Expected violation shape:

```lisp
(repo-tooling-violation unregistered-tool "scripts/<temporary-name>")
```

- [ ] **Step 6: Commit inventory + real enforcement**

```sh
git add knowledge/repo-tooling-inventory.lisp scripts/check-repo-tooling-inventory.lisp
git commit -m "feat(tooling): inventory active repo scripts in Lisp"
```

---

### Task 3: Add Guard navigation without duplicating inventory rows

**Files:**
- Modify: `knowledge/guard-reference.lisp`

**Interfaces:**
- Consumes: #382 registry/checker paths.
- Produces: Guard topic `repo-tooling` that points agents to the authoritative governance artifacts and verification command.

- [ ] **Step 1: Add one `reference` entry**

Add a topic alongside existing Guard governance/navigation topics:

```lisp
(reference
  (topic repo-tooling)
  (summary "Repo-owned tooling inventory and lifecycle metadata; navigation only, not language semantic authority")
  (authority (knowledge/repo-tooling-inventory.lisp
              docs/superpowers/specs/2026-09-17-repo-tooling-inventory-design.md))
  (how-to (read-repo-tooling-inventory run-check-repo-tooling-inventory-lisp))
  (verify (scripts/check-repo-tooling-inventory.lisp))
  (lifecycle current-governance)
  (provenance "my-lisp#382; inventory follows #76 Python migration and #299 Rust retirement valve")
  (unknown-route ask-agent))
```

Do not copy individual tool rows into Guard.

- [ ] **Step 2: Run Guard-related and tooling checks**

Run:

```sh
cargo run -p my-lisp-cli --bin my-lisp -- scripts/check-repo-tooling-inventory.lisp
cargo run -p my-lisp-cli --bin my-lisp -- scripts/authority-guard.lisp
```

Expected: tooling inventory exits 0; Guard source parses/evaluates without introducing a duplicate authority table.

- [ ] **Step 3: Commit Guard navigation**

```sh
git add knowledge/guard-reference.lisp
git commit -m "docs(guard): route repo tooling inventory"
```

---

### Task 4: Wire the Lisp checker into the existing cheap CI lanes

**Files:**
- Modify: `.github/workflows/ci.yml`

**Interfaces:**
- Consumes: `scripts/check-repo-tooling-inventory.lisp` and existing `my-lisp-cli` build path.
- Produces: PR and main CI fail closed on unregistered immediate tooling, without a new workflow or Rust glue.

- [ ] **Step 1: Add the checker to `pr-focused` after CLI is available**

Reuse the already-built CLI in the `Lisp-owned semantic authority guard` area or add a small neighboring step that builds once if needed:

```yaml
      - name: Lisp-owned repo tooling inventory
        shell: bash
        run: |
          set -euo pipefail
          cargo build -p my-lisp-cli --bin my-lisp
          ./target/debug/my-lisp scripts/check-repo-tooling-inventory.lisp
```

Do not add a new workflow.

- [ ] **Step 2: Add the same checker to `main-fast-critical`**

Place it after the existing `Machine non-interference guard` build step so `./target/debug/my-lisp` already exists:

```yaml
      - name: Repo tooling inventory
        run: ./target/debug/my-lisp scripts/check-repo-tooling-inventory.lisp
```

- [ ] **Step 3: Validate YAML syntax locally**

Run:

```sh
ruby -e 'require "psych"; Psych.parse_file(".github/workflows/ci.yml"); puts "ci.yml ok"'
```

Expected: `ci.yml ok`.

- [ ] **Step 4: Run the focused checker and existing Rust valve self-test**

Run:

```sh
cargo build -p my-lisp-cli --bin my-lisp
./target/debug/my-lisp scripts/check-repo-tooling-inventory.lisp
bash scripts/test-rust-one-way-valve.sh
```

Expected: inventory `(repo-tooling-ok)`; Rust valve self-test passes.

- [ ] **Step 5: Commit CI integration**

```sh
git add .github/workflows/ci.yml
git commit -m "ci: enforce Lisp-owned repo tooling inventory"
```

---

### Task 5: Final exact-head verification and #382 evidence

**Files:**
- Verify only; no planned production file changes.

**Interfaces:**
- Consumes: complete Task 1–4 branch head.
- Produces: evidence that #382 is GREEN without violating #299.

- [ ] **Step 1: Verify no Rust additions**

Against the implementation branch base, run:

```sh
git diff --numstat <base-sha>...HEAD -- '*.rs'
```

Expected: no added Rust lines and no new `.rs` paths. If any Rust addition exists, stop; #299 makes the branch RED regardless of purpose.

- [ ] **Step 2: Run focused repo-tooling verification**

```sh
cargo build -p my-lisp-cli --bin my-lisp
./target/debug/my-lisp scripts/check-repo-tooling-inventory.lisp
```

Expected: `(repo-tooling-ok)` and exit 0.

- [ ] **Step 3: Run cheap repository policy checks affected by this change**

```sh
bash scripts/test-rust-one-way-valve.sh
ruby -e 'require "psych"; Psych.parse_file(".github/workflows/ci.yml"); puts "ci.yml ok"'
git diff --check <base-sha>...HEAD
```

All must pass.

- [ ] **Step 4: Run the existing main fast governance path that does not require new Rust**

```sh
cargo run -p xtask -- verify
```

This is an unchanged existing verifier; #382 does not modify it. Expected: GREEN on the branch head or any unrelated pre-existing failure must be recorded exactly rather than hidden.

- [ ] **Step 5: Update #382 with exact evidence**

Post the implementation branch/PR head plus:

```text
repo tooling checker: GREEN
current immediate scripts coverage: 42/42 (41 pre-existing files + checker; scripts/tests excluded by explicit scope)
Python migration ownership: #317/#351/#76 as applicable
new Rust lines/files: 0
CI YAML parse: GREEN
Rust one-way valve self-test: GREEN
```

If concurrent work changes the immediate `scripts/` set before merge, rebase/refresh the registry and report the new exact count rather than forcing the stale 42 count.

- [ ] **Step 6: Commit any evidence-only adjustment if the repository convention requires one**

If no checked-in evidence file is required, do not invent one. The GitHub issue/PR check record is sufficient for this slice.
