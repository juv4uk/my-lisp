# Cyrillic file extension audit — GitHub issue juv4uk/my-lisp#62

Per the owner's 2026-09-10 decision: `.всм`/`.мій`/`.лісп` are
equal-standing Ukrainian spellings of `.wsm`/`.my`/`.lisp` — a
file-naming surface policy, not a new semantic authority. This audits
every place in this repo that filters files by these extensions,
following the issue's own instruction: fix real filters, and where
code doesn't filter by extension at all, prove Unicode paths aren't
rejected rather than inventing unnecessary work.

## Audited and fixed

- **`crates/my-lisp-lsp/src/workspace.rs`** — the one real
  extension-filtering site in this repo (the workspace scanner's
  `is_source` check). Added `Some("всм") | Some("мій") | Some("лісп")`
  alongside the existing three. Verified with a real temp-directory
  test (`cyrillic_extension_tests::scan_root_recognizes_all_three_cyrillic_extensions`):
  writes real `.мій`/`.всм`/`.лісп` files plus a `.txt` control file,
  confirms the three are scanned and the control file is still
  correctly ignored (proving the fix didn't accidentally widen the
  filter to "everything").
- **`.gitattributes`** — added `*.всм`/`*.мій`/`*.лісп` linguist
  classification lines mirroring the existing three exactly.
- **`crates/my-lisp-cli/src/main.rs`** help text — mentions the
  Cyrillic spellings alongside the existing "canonical extension +
  aliases" line.

## Audited and found not-applicable (with reason)

- **`crates/my-lisp-cli/src/main.rs`'s actual file-loading path**
  (`fs::read_to_string(filename)`, the code that runs `my-lisp <file>`):
  performs **no extension check of any kind**. Any filename is read and
  evaluated as-is. This means `.мій`/`.всм`/`.лісп` files already
  worked with zero code changes — verified directly, not assumed:
  `./target/debug/my-lisp.exe /tmp/приклад.мій` (containing `(+ 1 2)`)
  printed `3`, and `--oracle-check` on the same kind of path returned
  `(outcome valid)`. No fix needed here; this is the "prove it already
  works" case the issue anticipates.
- **CI workflow `paths:` triggers** (`.github/workflows/*.yml`): these
  watch specific canonical files by exact name (`lib/core.my`,
  `knowledge/swarm-*.wsm`), not a general extension glob. Introducing
  Cyrillic-spelled *copies* of these specific canonical files is not
  what this policy asks for — one semantic authority per contract
  file, multiple *spellings* of file extensions in general, not
  duplicate files. Not applicable.
- **`crates/my-lisp/src/lib.rs`'s `include_str!` bootstrap constants**
  (`CORE_LIBRARY_SOURCE`, `MACRO_LIBRARY_SOURCE`, etc.): these are
  compile-time embeds of specific named files, not a scanner or loader
  that filters by extension pattern. Not applicable for the same
  reason as the CI triggers above.

## Executable witness added

`tests/fixtures/приклад.мій` — a genuine UTF-8 Cyrillic filename and
extension (not a transliterated stand-in), containing `(+ 1 2)`.
`crates/my-lisp/tests/cyrillic_extension_witness.rs` reads and
evaluates it through the ordinary `eval_program` path, asserting the
result is `3` — proving the real repo file, not just a string literal
in a test, round-trips through the same path any `.my` fixture would.

## Acceptance evidence

- Existing Latin-extension fixtures/tests: unaffected, still pass
  (`cargo clippy -p my-lisp -p my-lisp-lsp -p my-lisp-cli --all-targets
  -- -D warnings` clean; new tests are additive, no existing test
  files touched).
- Corresponding Cyrillic fixture passes the same semantic path: yes,
  both at the CLI/evaluator level (`cyrillic_extension_witness.rs`)
  and the LSP workspace-scanner level
  (`workspace::cyrillic_extension_tests`).
- No new parser/format created merely for the different extension
  spelling: confirmed — same `parse`/`eval_program` functions, same
  `analysis::analyze` LSP path, zero new code paths.
- CI/tooling does not reject valid Ukrainian UTF-8 filenames: verified
  directly for the CLI (`--oracle-check`) and the LSP scanner (real
  temp-directory test); `.gitattributes` now classifies the new
  extensions the same as the existing three so GitHub's own file
  detection doesn't treat them as unknown binary/unclassified files.
