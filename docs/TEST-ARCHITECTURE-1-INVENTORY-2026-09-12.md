# TEST-ARCHITECTURE-1 — Step 1 Inventory (2026-09-12)

Classification-only pass over the test suite per the owner's review. **No tests were
merged or deleted in this step.** Findings below feed steps 2-5 (surface compression,
meta-corpus unification, policy-test extraction, deletion pass).

Guiding question applied to every test: *what specific mutation does this test kill
that no cheaper test also kills?*

## 1. Surface/UK cluster

- `cyrillic_extension_witness.rs` — confirmed stale historical-policy-witness (the
  file's own doc comment admits it makes an already-true fact executable, not a
  regression guard). **Delete candidate**, but only after the `.lisp` migration
  (issue #81) is far enough along that a generic "same source, three extensions,
  same result" test replaces it, per the owner's own suggestion.
- **Live inconsistency found**: `rivnopravnist_mov.rs` and `runtime_peer_operators.rs`
  assert executable authority no longer reads `lib/surface/uk-sa-coverage.wsm` (the
  legacy EN-shaped table), but `uk_surface_equivalence.rs` — the single strongest
  canonical equivalence test, and the natural merge target for the ~45 duplicate
  per-operator tests in `uk_sa_surface.rs`/`uk_sa_batch2.rs` — still parses that same
  legacy file as its data source. Must migrate `uk_surface_equivalence.rs` to read
  `semantic-registry.wsm` directly before merging other tests into it.
- The number **140** (stable UK surface count) is hardcoded independently in at least
  4 places across 2 files (`uk_surface_equivalence.rs` x2, `ukrainian_api_docs.rs` x2),
  plus related counts (`161`, `21`, `136`, `42`, `98`, `30`, `1`) hardcoded elsewhere —
  all will silently rot as the registry grows. Should derive once from the registry.
- `uk_sa_surface.rs` (~28 of its ~30 tests) and `uk_sa_batch2.rs` (~15 of its ~17 tests)
  are near-total duplicates of `uk_surface_equivalence.rs`'s registry-driven sweep —
  strongest merge/delete candidates, but a few names may not yet be in the stable
  registry and need coverage verification before deletion, not blind removal.
- `peer_surface_identity.rs`, `runtime_peer_operators.rs`, and `rivnopravnist_mov.rs`
  each independently implement the same "shadowing one peer name doesn't retarget
  others" and "surface files don't rebuild peers via symbols" checks with 3 different
  hardcoded name lists — one parametrized/registry-driven test would replace all three.
- `uk_surface_equivalence.rs`'s two keyboard/no-Latin-characters tests, and
  `ukrainska_programa_pryimannya.rs`'s equivalent, are text-lint, not semantic tests —
  move out of `cargo test` into a lint/policy tool.
- `semantic_form_identity.rs`, `uk_surface_inventory.rs`, `translation_boundary.rs`,
  `time_host_surface.rs`, `crates/my-lisp-host/tests/process_surface.rs` are
  well-scoped, non-redundant, and out of scope for this consolidation (some were
  name-collision false positives, e.g. "translation"/"surface" meaning something
  unrelated to human-language surfaces in those files).

## 2. Meta-eval cluster (15 files, 883 lines, ~67 tests)

- **At least 9 of 15 files compare meta output to native output instead of both to
  an independent corpus `expected`**: `meta_eval_advice_taker.rs`,
  `meta_eval_closure_parity.rs`, `meta_eval_error_provenance.rs`,
  `meta_eval_evidence.rs` (majority of its 12 tests), `meta_eval_later_binding.rs`
  (all 6 tests), `meta_eval_mutual.rs`, `meta_eval_parity.rs` (labels native "(oracle)"
  in its own panic message), and `meta_eval_semantic_registry.rs` (2 of 3 tests).
  `meta_eval_errors.rs` looks like this pattern on the surface but is actually correct
  (its `via_native` is a hand-supplied literal, not a live native read).
- `meta_eval_parity.rs`'s `IN_SCOPE_EXPRS` is the exact hand-copy-then-verify
  bureaucracy the owner flagged: 25 expressions typed by hand, then a second test
  re-parses `conformance.my` just to confirm the hand-copy was accurate.
- **`tests/fixtures/conformance.my` has no positive meta-eval-scope tag today** —
  only the negative `meta-eval-gap` tag exists. A new tag (e.g. `meta-eval` with a
  `status` of `required`/`supported`, mirroring the existing `wsm-native` nested-alist
  precedent) must be introduced before any runner can filter the corpus directly,
  per `scripts/fixtures-for-tier.my`'s existing tag-filtering pattern.
- 9 files share near-identical `meta_session`/`meta_eval_program`/`native_value`
  helper boilerplate — natural candidates to collapse into one corpus-driven runner:
  `meta_eval_parity.rs`, `meta_eval_evidence.rs`, `meta_eval_environment_semantics.rs`,
  `meta_eval_later_binding.rs`, `meta_eval_mutual.rs`, `meta_eval_error_kind_parity.rs`,
  `meta_eval_error_provenance.rs`, `meta_eval_error_detail_boundary.rs`,
  `meta_eval_closure_parity.rs`.
- `meta_eval.rs`, `meta_eval_advice_taker.rs`, `meta_eval_empty_program.rs` exercise
  genuinely distinct rigs and can stay as separate files (oracle-direction fixes
  still needed in the first two).
- `meta_eval_evidence_matrix.rs` shells out to `python3` twice
  (`check-meta-eval-evidence.py`, `generate-meta-eval-evidence.py --check`) — not a
  semantic test, move out of `cargo test` entirely.

## 3. Policy/docs/CI cluster

- No `xtask` crate or `my-lisp verify` subcommand exists yet — the relocation target
  must be created, likely wrapping/replacing the existing
  `scripts/check-meta-eval-evidence.py`, `scripts/generate-meta-eval-evidence.py`,
  `scripts/semantic-ownership.py` scripts.
- `documentation_contract.rs`'s `public_docs_share_current_project_identity_and_extension`
  is a live example of the exact failure mode the owner is worried about: it currently
  passes while [README.md:287](../README.md) still says `.wsm` is canonical and
  [docs/language-core.md:11](language-core.md) says `.lisp` is canonical — the test
  only checks all three extension tokens are *mentioned*, not which one is declared
  canonical, so it cannot catch this contradiction. Must fix README.md's claim
  regardless of where this test ends up living.
- `meta_eval_evidence_matrix.rs`, `semantic_ownership.rs` — pure python3-shell-out
  docs-drift checks, move out of `cargo test`.
- `typed_buffer_proposal.rs`, and two tests in `swarm_deprecation.rs` — real
  invariants worth keeping, but they validate governance/contract metadata, not
  runtime behavior; relocate to a future canon/contract validator rather than delete.
- `ukrainian_api_docs.rs` (7 of 11 tests), `swarm_deprecation.rs` (2 of 4 tests),
  `meta_eval_error_detail_boundary.rs` (1 of 3 tests) each mix a couple of pure
  markdown/doc-text checks in among otherwise-legitimate behavior tests — split
  rather than move/delete the whole file.
- `program_surface_translation.rs` genuinely tests behavior (three-surface parity of
  translator output) despite its `python3` dependency — keep, but its undocumented
  environment coupling is a hardening item, not a relocation candidate.

## Proposed order for steps 2-5

1. Fix the README.md canonical-extension claim (`.wsm`→`.lisp`) — small, unblocks the
   `documentation_contract.rs` fix and removes a real live contradiction independent
   of any test refactor.
2. Migrate `uk_surface_equivalence.rs` off `uk-sa-coverage.wsm` onto
   `semantic-registry.wsm` directly (fixes the live inconsistency), then merge
   `uk_sa_surface.rs`/`uk_sa_batch2.rs`'s redundant per-operator tests into it,
   verifying no name loses coverage. Delete `cyrillic_extension_witness.rs` once a
   generic multi-extension witness exists.
3. Introduce a `meta-eval` corpus tag in `conformance.my`, build one data-driven
   runner reading it, migrate the 9 duplicate-boilerplate meta-eval files onto it,
   fixing oracle direction (both native and meta checked against `expected`,
   never against each other) as part of the same move.
4. Create the `xtask`/`verify` home for docs-drift/policy checks, move the
   identified pure-doc/python3-shell-out tests there, split the mixed files.
5. Deletion pass only after 2-4 land, with an explicit note per deleted test of
   which surviving test kills the same mutation.
