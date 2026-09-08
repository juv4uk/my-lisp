#!/usr/bin/env bash
# Запустити один раз на клон: активує versioned pre-commit hook
# (githooks/pre-commit), зокрема check-bilingual-docs.
set -euo pipefail

repo_root="$(git rev-parse --show-toplevel)"
cd "$repo_root"
git config core.hooksPath githooks
chmod +x githooks/pre-commit scripts/check-bilingual-docs
echo "OK: core.hooksPath -> githooks (pre-commit тепер запускає check-bilingual-docs)"
