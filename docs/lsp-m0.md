# my-lisp-lsp — Language Server Protocol adapter (M0–M2)

> A thin LSP adapter over the canonical my-lisp core.
> The LSP never re-parses `.my`, never greps for definitions, never
> invents semantics.

## Boundary rules (enforced by module layout)

| Concern | Owner |
|---|---|
| Parsing, spans, errors | `crates/my-lisp` — canonical `parse()`, `Expr{kind,span}`, `LanguageError` reused unchanged |
| Position mapping (byte ↔ line/UTF-16) | LSP adapter (`analysis.rs`) — pure arithmetic |
| JSON-RPC framing (stdio Content-Length) | LSP transport (`transport.rs`) |
| JSON-RPC **decode** | canonical `my_lisp::parse_json` (extracted from the `json-parse` special form; the special form delegates to it) |
| JSON-RPC **encode** | LSP transport-local (`jsonout.rs`) — deliberately NOT in the language core: no my-lisp feature independently needs a canonical serializer |

There is exactly one parser and one JSON decoder. Nothing is duplicated.

## M0 scope

Implemented: `initialize` (capabilities list exactly what exists),
`textDocument/didOpen`, `textDocument/didChange` (full sync),
`publishDiagnostics` (pushed on sync), `textDocument/documentSymbol`,
`textDocument/hover`, `textDocument/definition`, shutdown/exit lifecycle.

Out of scope: completion, rename, formatting, references, semantic
tokens, workspace indexing, AI/LLM and Yantra integration.

## Behavior notes

- **Diagnostics** come only from the canonical parser (`LanguageError`
  with its proven span). Valid documents produce an empty list; nothing
  is invented beyond parse-time semantics.
- **documentSymbol** returns only structurally provable definitions:
  top-level `(def name ...)` / `(defmacro name ...)` whose second element
  is a symbol. Symbol text inside strings or comments can never qualify —
  it does not exist as a def-form in the AST. `SymbolKind` is Function
  (12) because LSP has no Macro kind; the exact keyword travels in
  `detail`. `selectionRange` covers just the defined name.
- **hover / definition** resolve through the same parse-tree def table,
  same document only (M0). Hovering unknown names returns `null` —
  unknown stays unknown.

## Running

Two equivalent entrypoints share one implementation:

```bash
cargo build --workspace

# subcommand of the main CLI binary (preferred):
target/debug/my-lisp lsp

# standalone binary (kept for now):
target/debug/my-lisp-lsp
```

Both speak LSP over stdio (`Content-Length` framing). stdout carries only
framed protocol traffic; use stderr for any debug logging. Point your
editor's LSP config at `my-lisp lsp`.

## Release integration

`scripts/release.my` bumps all five canonical crates together
(`my-lisp`, `my-lisp-cli`, `my-lisp-literate`, `my-lisp-wasm`,
`my-lisp-lsp`). The test
`crates/my-lisp-lsp/tests/release_parity.rs` fails on version drift,
so a crate added to one list but not the other cannot slip through.

## Tests

- `crates/my-lisp-lsp/tests/e2e.rs` — nine end-to-end tests driving the
  real server with real JSON-RPC messages: capabilities, false-positive
  diagnostics, real parser-backed diagnostic ranges, def/defmacro symbols
  with exact selection ranges, hover payload with canonical source,
  go-to-definition range, string/comment non-detection, malformed-input
  robustness (server keeps serving afterwards).
- `crates/my-lisp-lsp/tests/stdio.rs` — spawns the actual binary and
  performs a framed initialize → initialized → shutdown → exit handshake
  over pipes, like an editor would.

## M1 scope (2026-08-22)

Added on the same principles (canonical parser only, nothing invented):

- **Workspace index** (`workspace.rs`): `initialize` with `rootUri` scans
  all `.my` files under the root (4 MB per-file cap, hidden dirs skipped)
  and remembers every structurally proven definition with its file URI.
  Open/change events refresh one document's contributions incrementally.
- **Cross-file go-to-definition**: same-document resolution first (M0 path),
  then the workspace index. Ranges come from the defining file's own text.
- **Completion** (`textDocument/completion`, kind Function): builtins
  (static snapshot of the core's match arms — see docs/FUNCTIONS.md),
  local defs, workspace defs; filtered by the symbol prefix at the cursor.

## Known limitations

- Diagnostics are parse-only; eval-time errors are not reported yet.
- Full-document sync (simple and correct, not incremental).
- Completion inherits parse-only honesty: a document that fails to parse
  contributes no definitions (visible while typing unbalanced forms).
- The builtin completion list is a static snapshot duplicated from the
  core's match arms; after contract 2.1 it should be derived from the
  environment ((env)) instead.

## M2 scope (2026-08-22)

- **References** (`textDocument/references`): every code occurrence of
  the cursor symbol across open + indexed documents.
  `includeDeclaration` (context) is honored per-document — declarations
  live wherever their def-form is, not only in the cursor's file.
- **Rename** (`textDocument/rename`): WorkspaceEdit across all affected
  files. newName validated against the my-lisp symbol charset (error
  -32602 otherwise). Quoted data (`(quote x)` subtrees) is never touched:
  data symbols are not code references, per analysis::symbol_occurrences.
