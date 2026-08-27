#!/usr/bin/env bash
# gen-docs-index.sh — MYLISP-DOC-LIFECYCLE-METADATA's "generated navigation,
# not hand-maintained" half. Scans docs/*.md for each file's own first
# **Статус:**/**Status:** line and prints a table -- run this, don't hand-copy
# a snapshot into a committed file that then drifts (the exact failure
# MYLISP-DOCS-DRIFT-SWEEP already found and fixed once in docs/testing.md).
#
# Usage: scripts/gen-docs-index.sh   (from repo root, or anywhere -- resolves
# its own location first)
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DOCS_DIR="$ROOT/docs"

printf '%-58s | %s\n' "file" "status (first line found, verbatim)"
printf '%s\n' "-----------------------------------------------------------|------------------------------------------------------------"

# Sorted for deterministic output -- a generated index that reorders
# itself between runs is its own small drift problem.
while IFS= read -r -d '' f; do
    rel="${f#"$ROOT"/}"
    status_line=$(grep -m1 -E '^\*\*(Статус|Status):' "$f" 2>/dev/null || true)
    if [ -z "$status_line" ]; then
        status_line="UNSPECIFIED — no **Статус:**/**Status:** line found"
    else
        # Strip the leading "**Статус:**"/"**Status:**" marker itself,
        # keep the rest verbatim (including any embedded date/author).
        status_line=$(printf '%s\n' "$status_line" | sed -E 's/^\*\*(Статус|Status):\*\*[[:space:]]*//')
    fi
    printf '%-58s | %s\n' "$rel" "$status_line"
done < <(find "$DOCS_DIR" -maxdepth 1 -name '*.md' -print0 | sort -z)
