#!/usr/bin/env bash
set -euo pipefail

# #231 / #112-#114: the fast semantic lane executes current Lisp-owned
# expectations through host observers. It deliberately does not run legacy
# host-authored Canon truth assertions; deep/current-contract lanes retain
# broader mechanism and integration evidence.
cargo test -p my-lisp \
  --test witness_authority \
  --test structural_query_inventory \
  --test structural_observation_contract \
  --test deep_structural_relation_contract \
  --test exact_q_binary_contract \
  --test mathematical_result_taxonomy \
  --test control_dispatch_contract \
  --test canon_laws_v2_contract \
  --test reason_honesty_contract \
  --test unification_outcome_contract \
  --test knowledge_clause_kind_contract \
  --test content_store_authority \
  --test decimal_comma_authority \
  --test authority_guard_contract \
  --test semantic_ref_fail_closed

# #291: quantity semantics live in Lisp. The shell observes only the named
# pass envelope; expected scientific quantities and relations stay in the
# Lisp witness itself. No replacement Rust observer is introduced.
quantity_status="$(cargo run --quiet -p my-lisp-cli --bin my-lisp -- tests/fixtures/exact-quantity-arithmetic-witness.lisp)"
if [[ "$quantity_status" != "(exact-quantity-arithmetic-witness (status pass))" ]]; then
  printf 'exact quantity Lisp witness failed: %s\n' "$quantity_status" >&2
  exit 1
fi

# #305 / TASK-001: empty meta-program semantics are owned by Lisp. The shell
# observes only the named pass envelope; environment/result meaning stays in
# the witness and no replacement Rust semantic assertion is introduced.
meta_empty_status="$(cargo run --quiet -p my-lisp-cli --bin my-lisp -- tests/fixtures/meta-eval-empty-program-witness.lisp)"
if [[ "$meta_empty_status" != "(meta-eval-empty-program-witness (status pass))" ]]; then
  printf 'empty meta-program Lisp witness failed: %s\n' "$meta_empty_status" >&2
  exit 1
fi

# #329 / #220: meta-evaluator peer-surface semantics are authored in Lisp.
# The shell checks only the named envelope; Canon/necessary-form expectations
# and current structural result shapes remain inside the witness.
meta_peer_status="$(cargo run --quiet -p my-lisp-cli --bin my-lisp -- tests/fixtures/meta-eval-peer-surface-witness.lisp)"
if [[ "$meta_peer_status" != "(meta-eval-peer-surface-witness (status pass))" ]]; then
  printf 'meta-eval peer-surface Lisp witness failed: %s\n' "$meta_peer_status" >&2
  exit 1
fi
