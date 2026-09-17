#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 1 ]]; then
  echo "usage: $0 <repo-relative-path>" >&2
  exit 2
fi

case "$1" in
  tests/fixtures/canon-zero-v[0-9]*.lisp|
  tests/fixtures/control-dispatch-v[0-9]*.lisp|
  tests/fixtures/deep-structural-relation-v[0-9]*.lisp|
  tests/fixtures/exact-q-binary-v[0-9]*.lisp|
  tests/fixtures/knowledge-clause-kind-v[0-9]*.lisp|
  tests/fixtures/mathematical-result-v[0-9]*.lisp|
  tests/fixtures/reason-honesty-v[0-9]*.lisp|
  tests/fixtures/reason-module-honesty-v[0-9]*.lisp|
  tests/fixtures/reason-observe-honesty-v[0-9]*.lisp|
  tests/fixtures/structural-observation-v[0-9]*.lisp|
  tests/fixtures/structure-core-v[0-9]*.lisp|
  tests/fixtures/unification-outcome-v[0-9]*.lisp)
    exit 0
    ;;
  *)
    exit 1
    ;;
esac
