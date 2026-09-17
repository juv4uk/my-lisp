#!/usr/bin/env bash
set -euo pipefail

repo_root=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
guard="$repo_root/scripts/rust-one-way-valve.sh"
workflow="$repo_root/.github/workflows/ci.yml"

fail() {
  echo "rust-one-way-valve self-test: $*" >&2
  exit 1
}

[[ -f "$guard" ]] || fail "RED: missing scripts/rust-one-way-valve.sh"
grep -Fq 'bash scripts/rust-one-way-valve.sh "$BASE_SHA" "$HEAD_SHA"' "$workflow" \
  || fail "CI does not invoke the #300 valve on the PR base/head diff"

tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT
repo="$tmp/repo"
mkdir -p "$repo"
cd "$repo"

git init -q
git config user.name rust-valve-self-test
git config user.email rust-valve-self-test@example.invalid
mkdir -p src lib
cat > src/existing.rs <<'EOF'
fn keep() {}
fn removable() {}
EOF
cat > src/mechanism.rs <<'EOF'
fn transport() {}
EOF
cat > src/old.rs <<'EOF'
fn old_a() {}
fn old_b() {}
EOF
cat > src/replacement.rs <<'EOF'
fn existing_replacement_surface() {}
EOF
cat > lib/core.lisp <<'EOF'
(quote baseline)
EOF

git add -A
git commit -qm baseline
base=$(git rev-parse HEAD)

reset_case() {
  git reset --hard -q "$base"
  git clean -fdq
}

commit_case() {
  git add -A
  git commit -qm "$1"
  case_head=$(git rev-parse HEAD)
}

expect_red() {
  local name="$1"
  local path="$2"
  local log="$tmp/$name.log"
  if bash "$guard" "$base" "$case_head" >"$log" 2>&1; then
    cat "$log" >&2
    fail "$name unexpectedly passed"
  fi
  grep -Fq "$path" "$log" || {
    cat "$log" >&2
    fail "$name did not name offending path $path"
  }
  grep -Fq '#299' "$log" || {
    cat "$log" >&2
    fail "$name diagnostic did not link #299"
  }
}

expect_green() {
  local name="$1"
  local log="$tmp/$name.log"
  if ! bash "$guard" "$base" "$case_head" >"$log" 2>&1; then
    cat "$log" >&2
    fail "$name unexpectedly failed"
  fi
  grep -Fq 'RUST-ONE-WAY-VALVE OK' "$log" || {
    cat "$log" >&2
    fail "$name did not emit the GREEN verdict"
  }
}

# RED: executable Rust growth in an existing file.
reset_case
printf '\nfn added_behavior() {}\n' >> src/existing.rs
commit_case red-executable-rust
expect_red red-executable-rust src/existing.rs

# RED: comments count as Rust growth too.
reset_case
printf '\n// newly added Rust comment\n' >> src/existing.rs
commit_case red-rust-comment
expect_red red-rust-comment src/existing.rs

# RED: even an empty new .rs path is forbidden.
reset_case
: > src/new.rs
commit_case red-new-rust-path
expect_red red-new-rust-path src/new.rs
grep -Fq 'new=yes' "$tmp/red-new-rust-path.log" || fail "new Rust path diagnostic did not report new=yes"

# RED: deleting old Rust does not compensate for adding replacement Rust elsewhere.
reset_case
sed -i '/old_b/d' src/old.rs
printf '\nfn replacement_growth() {}\n' >> src/replacement.rs
commit_case red-delete-and-replace
expect_red red-delete-and-replace src/replacement.rs

# RED: mechanism/ABI/transport labels are not exemptions.
reset_case
printf '\nfn new_transport_mechanism() {}\n' >> src/mechanism.rs
commit_case red-mechanism-growth
expect_red red-mechanism-growth src/mechanism.rs

# RED: a rename creates a new .rs path and cannot bypass the valve.
reset_case
git mv src/old.rs src/renamed.rs
commit_case red-rust-rename
expect_red red-rust-rename src/renamed.rs

# GREEN: deleting Rust lines only.
reset_case
sed -i '/removable/d' src/existing.rs
commit_case green-line-deletion
expect_green green-line-deletion

# GREEN: deleting an obsolete Rust file entirely.
reset_case
git rm -q src/old.rs
commit_case green-file-deletion
expect_green green-file-deletion

# GREEN: Lisp may grow while Rust shrinks.
reset_case
printf '(quote preserved-law)\n' >> lib/core.lisp
sed -i '/removable/d' src/existing.rs
commit_case green-lisp-growth-rust-shrink
expect_green green-lisp-growth-rust-shrink

# GREEN: non-Rust-only changes do not trip the outer valve.
reset_case
printf '(quote non-rust-only)\n' >> lib/core.lisp
commit_case green-non-rust-only
expect_green green-non-rust-only

echo "rust-one-way-valve self-test: PASS"
