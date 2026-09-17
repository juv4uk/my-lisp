#!/usr/bin/env bash
set -euo pipefail

# #333: read-file-bytes is a host mechanism that returns a proper Lisp list
# constructed directly from Vec<u8>, with every element marked Exactness::Exact.
# `read-file` may therefore consume that already-proven byte domain directly in
# the Lisp-owned UTF-8 decoder worker. Generic `utf8-decode` remains responsible
# for validating arbitrary user-provided lists.
fs_surface="lib/fs.lisp"

if grep -Fq '(utf8-decode-string (read-file-bytes path))' "$fs_surface"; then
  echo 'read-file still repeats generic byte-domain validation after read-file-bytes' >&2
  exit 1
fi

if ! grep -Fq '(utf8-decode-onto (read-file-bytes path) (quote ()))' "$fs_surface"; then
  echo 'read-file must enter the Lisp UTF-8 decoder worker at the proven-byte boundary' >&2
  exit 1
fi
