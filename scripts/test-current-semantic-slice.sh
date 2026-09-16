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
  --test exact_q_binary_contract \
  --test authority_guard_contract \
  --test semantic_ref_fail_closed
