#!/usr/bin/env bash
set -euo pipefail

if (($# != 2)); then
  echo "usage: $0 BASE_SHA HEAD_SHA" >&2
  exit 2
fi

base=$1
head=$2

git rev-parse --verify "${base}^{commit}" >/dev/null
git rev-parse --verify "${head}^{commit}" >/dev/null

merge_base=$(git merge-base "$base" "$head")

changed=$(git diff --no-renames --numstat "$merge_base" "$head" -- '*.rs' || true)

if [[ -n "$changed" ]]; then
  echo 'RUST-GROWTH-ALLOWED: Rust source changed.'
  echo "$changed"
else
  echo 'RUST-GROWTH-ALLOWED: no Rust source changes.'
fi

echo 'Rust growth is permitted for substrate/mechanism work. Semantic authority remains subject to architecture/conformance review. See #299, #695, #706.'
