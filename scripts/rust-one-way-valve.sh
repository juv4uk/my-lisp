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

declare -A seen=()
declare -A is_new=()
declare -A additions=()

while IFS=$'\t' read -r status path; do
  [[ -n "${path:-}" ]] || continue
  seen["$path"]=1
  if [[ "$status" == "A" ]]; then
    is_new["$path"]=yes
  fi
done < <(git diff --no-renames --name-status "$base" "$head" -- '*.rs')

while IFS=$'\t' read -r added deleted path; do
  [[ -n "${path:-}" ]] || continue
  seen["$path"]=1
  if [[ "$added" == "-" ]]; then
    additions["$path"]=binary
  else
    additions["$path"]=$added
  fi
done < <(git diff --no-renames --numstat "$base" "$head" -- '*.rs')

violations=0
for path in "${!seen[@]}"; do
  added=${additions[$path]:-0}
  new=${is_new[$path]:-no}
  bad=0

  if [[ "$new" == "yes" ]]; then
    bad=1
  elif [[ "$added" == "binary" ]]; then
    bad=1
  elif [[ "$added" =~ ^[0-9]+$ ]] && ((added > 0)); then
    bad=1
  fi

  if ((bad)); then
    printf 'RUST-ONE-WAY-VALVE RED: %s added=%s new=%s. Rust may only shrink. See #299.\n' \
      "$path" "$added" "$new" >&2
    violations=1
  fi
done

if ((violations)); then
  exit 1
fi

echo 'RUST-ONE-WAY-VALVE OK: Rust did not grow (#299).'
