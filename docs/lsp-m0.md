# my-lisp-lsp — Language Server Protocol adapter (M0)

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

```bash
cargo build -p my-lisp-lsp
# point your editor at: target/debug/my-lisp-lsp  (stdio transport)
```

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

## Known limitations

- Diagnostics are parse-only; eval-time errors are not reported yet.
- Full-document sync (simple and correct, not incremental).
- Same-document definitions only; no workspace-wide index.
