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
  --test authority_guard_contract \
  --test semantic_ref_fail_closed

# Mechanism-honesty witness for #250. It is intentionally narrow: semantic
# authority remains in Lisp, while this unit slice proves that evaluator
# capability transport does not collapse an unreadable registry into absence.
cargo test -p my-lisp --lib eval::capabilities::honesty_tests
