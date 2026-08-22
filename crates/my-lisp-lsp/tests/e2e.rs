//! MY-LISP-LSP M0 acceptance tests. Every test drives the real server
//! with real framed JSON-RPC/LSP messages and asserts on the protocol
//! responses — a handler only counts as implemented when this passes.

use my_lisp_lsp::Harness as Server;

fn raw(m: &str) -> String { m.to_string() }

/// initialize → capabilities must list exactly the M0 features.
#[test]
fn t01_initialize_returns_m0_capabilities() {
    let mut server = Server::new();
    let replies = server.feed(&[raw(r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#)]);
    assert_eq!(replies.len(), 1);
    let r = replies[0].as_str();
    assert!(r.contains("\"capabilities\""), "{r}");
    assert!(r.contains("\"textDocumentSync\":1"), "{r}");
    assert!(r.contains("\"documentSymbolProvider\":true"), "{r}");
    assert!(r.contains("\"hoverProvider\":true"), "{r}");
    assert!(r.contains("\"definitionProvider\":true"), "{r}");
    // Out-of-scope features must NOT be advertised.
    assert!(!r.to_lowercase().contains("completion"), "{r}");
    assert!(!r.to_lowercase().contains("rename"), "{r}");
}

const VALID_DOC: &str = "; a comment mentioning mystery_word\n(def answer 42)\n(defmacro unless (cond body) (list 'if cond body))\n";

/// didOpen of a valid document → publishDiagnostics with EMPTY list.
#[test]
fn t02_valid_document_produces_no_false_diagnostics() {
    let mut server = Server::new();
    let open = format!(
        r#"{{"jsonrpc":"2.0","method":"textDocument/didOpen","params":{{"textDocument":{{"uri":"file:///t.my","languageId":"my-lisp","version":1,"text":{}}}}}}}"#,
        json_string(VALID_DOC)
    );
    let replies = server.feed(&[open]);
    assert_eq!(replies.len(), 1);
    let r = replies[0].as_str();
    assert!(r.contains("publishDiagnostics"), "{r}");
    assert!(r.contains("\"diagnostics\":[]"), "no false diagnostics: {r}");
}

/// didOpen of an invalid document → at least one parser-backed diagnostic
/// with the canonical error span.
#[test]
fn t03_invalid_document_produces_real_diagnostic() {
    let mut server = Server::new();
    let bad = "(def x 1)\n(def broken\n"; // unclosed list on line 2
    let open = format!(
        r#"{{"jsonrpc":"2.0","method":"textDocument/didOpen","params":{{"textDocument":{{"uri":"file:///bad.my","languageId":"my-lisp","version":1,"text":{}}}}}}}"#,
        json_string(bad)
    );
    let replies = server.feed(&[open]);
    assert_eq!(replies.len(), 1);
    let r = replies[0].as_str();
    assert!(r.contains("publishDiagnostics"), "{r}");
    assert!(!r.contains("\"diagnostics\":[]"), "must contain a diagnostic: {r}");
    assert!(r.contains("\"severity\":1"), "{r}");
    assert!(r.contains("\"source\":\"my-lisp\""), "{r}");
    // The diagnostic range must point into line 1 (0-based), where the
    // unclosed form starts — proven by the canonical parser's span.
    assert!(r.contains("\"start\":{\"line\":1"), "{r}");
}

/// documentSymbol finds a real top-level `(def ...)` with exact ranges.
#[test]
fn t04_document_symbol_finds_def() {
    let (reply, _) = symbols_for(VALID_DOC);
    assert!(reply.contains("\"name\":\"answer\""), "{reply}");
    assert!(reply.contains("\"detail\":\"def\""), "{reply}");
    // selectionRange points at the NAME, not the whole form:
    // `(def answer` — the name starts at character 5.
    assert!(reply.contains("\"selectionRange\":{\"start\":{\"line\":1,\"character\":5}"), "{reply}");
    // ordinary text in the comment is not a symbol
    assert!(!reply.contains("mystery_word"), "{reply}");
}

/// documentSymbol finds a real `(defmacro ...)`.
#[test]
fn t05_document_symbol_finds_defmacro() {
    let (reply, _) = symbols_for(VALID_DOC);
    assert!(reply.contains("\"name\":\"unless\""), "{reply}");
    assert!(reply.contains("\"detail\":\"defmacro\""), "{reply}");
}

fn symbols_for(doc: &str) -> (String, String) {
    let mut server = Server::new();
    let open = format!(
        r#"{{"jsonrpc":"2.0","method":"textDocument/didOpen","params":{{"textDocument":{{"uri":"file:///s.my","languageId":"my-lisp","version":1,"text":{}}}}}}}"#,
        json_string(doc)
    );
    let symbols = r#"{"jsonrpc":"2.0","id":7,"method":"textDocument/documentSymbol","params":{"textDocument":{"uri":"file:///s.my"}}}"#;
    let replies = server.feed(&[open, symbols.to_string()]);
    assert_eq!(replies.len(), 2);
    (replies[1].clone(), replies[0].clone())
}

/// hover on a locally defined symbol → kind + location + canonical source.
#[test]
fn t06_hover_on_known_definition_is_useful() {
    let mut server = Server::new();
    let doc = "(def greeting \"hello\")\n(print greeting)\n";
    let open = open_msg("file:///h.my", doc);
    // hover over `greeting` usage on line 1 char 8.
    let hover = hover_msg("file:///h.my", 1, 8);
    let replies = server.feed(&[open, hover]);
    assert_eq!(replies.len(), 2);
    let r = &replies[1];
    assert!(r.contains("\"result\":{\"contents\":{\"kind\":\"markdown\""), "{r}");
    assert!(r.contains("**def** `greeting`"), "{r}");
    // The canonical representation of the defining form travels along.
    assert!(r.contains("(def greeting \\\"hello\\\")"), "{r}");
}

/// definition on a use of a local symbol → the def name's exact range.
#[test]
fn t07_definition_resolves_to_correct_range() {
    let mut server = Server::new();
    let doc = "(def target (+ 1 2))\n(+ target 10)\n";
    let open = open_msg("file:///d.my", doc);
    let goto = format!(
        r#"{{"jsonrpc":"2.0","id":3,"method":"textDocument/definition","params":{{"textDocument":{{"uri":"file:///d.my"}},"position":{{"line":1,"character":3}}}}}}"#
    );
    let replies = server.feed(&[open, goto]);
    assert_eq!(replies.len(), 2);
    let r = &replies[1];
    // `target` is defined at line 0, chars 5..11 ("(def |target| ...").
    assert!(
        r.contains("\"uri\":\"file:///d.my\"") && r.contains("\"range\":{\"start\":{\"line\":0,\"character\":5},\"end\":{\"line\":0,\"character\":11}}"),
        "{r}"
    );
}

/// Symbol text inside strings and comments must never be a definition.
#[test]
fn t08_symbols_in_strings_and_comments_are_not_definitions() {
    let mut server = Server::new();
    let doc = concat!(
        "; (def commented-out 1)\n",
        "(print \"(def in-string 2)\")\n",
        "(def real-def 3)\n"
    );
    let open = open_msg("file:///c.my", doc);
    let symbols = symbols_msg("file:///c.my");
    let replies = server.feed(&[open, symbols]);
    let r = &replies[1];
    assert!(r.contains("real-def"), "{r}");
    assert!(!r.contains("commented-out"), "comment text leaked as symbol: {r}");
    assert!(!r.contains("in-string"), "string text leaked as symbol: {r}");
}

/// Malformed input must not crash the server; later requests still work.
#[test]
fn t09_malformed_input_does_not_crash() {
    let mut server = Server::new();
    // garbage JSON
    let replies = server.feed(&["this is not json {{{".to_string()]);
    assert_eq!(replies.len(), 1);
    assert!(replies[0].contains("-32700"), "ParseError expected: {}", replies[0]);
    // unknown method with id → MethodNotFound; without id → silent drop
    let replies = server.feed(&[
        r#"{"jsonrpc":"2.0","id":9,"method":"textDocument/rename","params":{}}"#.to_string(),
        r#"{"jsonrpc":"2.0","method":"$/unknownNotification","params":{}}"#.to_string(),
        r#"{"jsonrpc":"2.0"}"#.to_string(),
    ]);
    assert_eq!(replies.len(), 1);
    assert!(replies[0].contains("-32601"), "{}", replies[0]);
    // server still fully functional afterwards
    let open = open_msg("file:///ok.my", "(def alive t)");
    let symbols = symbols_msg("file:///ok.my");
    let replies = server.feed(&[open, symbols]);
    assert!(replies[1].contains("\"name\":\"alive\""), "{}", replies[1]);
}

// ---------------------------------------------------------------------------
// helpers
// ---------------------------------------------------------------------------

pub fn json_string(s: &str) -> String {
    let mut out = String::from("\"");
    for ch in s.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

fn open_msg(uri: &str, text: &str) -> String {
    format!(
        r#"{{"jsonrpc":"2.0","method":"textDocument/didOpen","params":{{"textDocument":{{"uri":{uri},"languageId":"my-lisp","version":1,"text":{text}}}}}}}"#,
        uri = json_string(uri),
        text = json_string(text),
    )
}

fn symbols_msg(uri: &str) -> String {
    format!(
        r#"{{"jsonrpc":"2.0","id":99,"method":"textDocument/documentSymbol","params":{{"textDocument":{{"uri":{}}}}}}}"#,
        json_string(uri)
    )
}

fn hover_msg(uri: &str, line: u32, character: u32) -> String {
    format!(
        r#"{{"jsonrpc":"2.0","id":98,"method":"textDocument/hover","params":{{"textDocument":{{"uri":{}}},"position":{{"line":{line},"character":{character}}}}}}}"#,
        json_string(uri)
    )
}
