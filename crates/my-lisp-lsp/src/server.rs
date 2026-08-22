//! server.rs — LSP dispatch. Maps decoded JSON-RPC requests onto the
//! language analysis (`analysis.rs`, which itself is only the canonical
//! my-lisp parser) and emits protocol responses. Holds per-document state
//! for full-text sync.
//!
//! The handler set is exactly M0: initialize, didOpen, didChange,
//! publishDiagnostics (pushed on sync), documentSymbol, hover,
//! definition, plus shutdown/exit lifecycle. Anything else answers
//! MethodNotFound rather than pretending.

use crate::analysis::{self, offset_to_position, span_text};
use crate::jsonout::str_lit;
use crate::protocol::{
    self, as_array, as_i64, as_str, decode, get, publish_diagnostics, response, span_to_range,
};
use my_lisp::Value;
use std::collections::HashMap;

const PARSE_ERROR: i64 = -32700;
const INVALID_REQUEST: i64 = -32600;
const METHOD_NOT_FOUND: i64 = -32601;

#[derive(Default)]
pub struct Server {
    documents: HashMap<String, String>,
}

impl Server {
    pub fn new() -> Self {
        Self::default()
    }

    /// Process one raw JSON-RPC message; returns every outgoing message
    /// (responses and pushed notifications) in order, unframed. Never
    /// panics on malformed input — that is acceptance requirement 9.
    pub fn handle_message(&mut self, text: &str) -> Vec<String> {
        let incoming = match decode(text) {
            Ok(incoming) => incoming,
            Err(message) => {
                return vec![response(
                    &None,
                    None,
                    Some((PARSE_ERROR, format!("parse error: {message}"))),
                )];
            }
        };
        // A notification carries no id and expects no response, but a
        // message with an id AND no method is a malformed request.
        let Some(method) = incoming.method.clone() else {
            if incoming.id.is_some() {
                return vec![response(
                    &incoming.id,
                    None,
                    Some((INVALID_REQUEST, "missing method".into())),
                )];
            }
            return vec![];
        };
        match method.as_str() {
            "initialize" => vec![self.initialize(&incoming)],
            "initialized" | "$/cancelRequest" => vec![],
            "shutdown" => vec![response(&incoming.id, Some("null".to_string()), None)],
            "exit" => vec![], // the stdio loop stops via wants_exit()
            "textDocument/didOpen" => self.did_open(&incoming),
            "textDocument/didChange" => self.did_change(&incoming),
            "textDocument/documentSymbol" => vec![self.document_symbols(&incoming)],
            "textDocument/hover" => vec![self.hover(&incoming)],
            "textDocument/definition" => vec![self.definition(&incoming)],
            _ => {
                if incoming.id.is_some() {
                    vec![response(
                        &incoming.id,
                        None,
                        Some((METHOD_NOT_FOUND, format!("method not found: {method}"))),
                    )]
                } else {
                    vec![]
                }
            }
        }
    }

    /// Whether this raw message is the `exit` notification.
    pub fn wants_exit(text: &str) -> bool {
        matches!(decode(text), Ok(incoming) if incoming.method.as_deref() == Some("exit"))
    }

    fn initialize(&self, incoming: &protocol::Incoming) -> String {
        // Capabilities list exactly what M0 implements — nothing more.
        let capabilities = concat!(
            "{\"textDocumentSync\":1,", // 1 = Full: simplest correct sync for M0
            "\"documentSymbolProvider\":true,",
            "\"hoverProvider\":true,",
            "\"definitionProvider\":true}"
        );
        let result = format!(
            "{{\"capabilities\":{capabilities},\"serverInfo\":{{\"name\":\"my-lisp-lsp\",\"version\":\"0.1.0\"}}}}"
        );
        response(&incoming.id, Some(result), None)
    }

    fn text_document<'a>(params: &'a Option<Value>, key: &'a str) -> Option<&'a Value> {
        get(params.as_ref()?, key)
    }

    fn uri_and_text(
        params: &Option<Value>,
        documents: &HashMap<String, String>,
    ) -> Option<(String, String)> {
        let td = Self::text_document(params, "textDocument")?;
        let uri = as_str(get(td, "uri"))?.to_string();
        let text = documents.get(&uri)?.clone();
        Some((uri, text))
    }

    fn did_open(&mut self, incoming: &protocol::Incoming) -> Vec<String> {
        let Some(uri) = Self::text_document(&incoming.params, "textDocument")
            .and_then(|td| as_str(get(td, "uri")).map(str::to_string))
        else {
            return vec![];
        };
        let Some(text) = Self::text_document(&incoming.params, "textDocument")
            .and_then(|td| as_str(get(td, "text")).map(str::to_string))
        else {
            return vec![];
        };
        self.documents.insert(uri.clone(), text);
        vec![self.publish(&uri)]
    }

    fn did_change(&mut self, incoming: &protocol::Incoming) -> Vec<String> {
        let Some(uri) = Self::text_document(&incoming.params, "textDocument")
            .and_then(|td| as_str(get(td, "uri")).map(str::to_string))
        else {
            return vec![];
        };
        // Full sync: the LAST content change carries the whole document.
        let changes = as_array(incoming.params.as_ref().and_then(|p| get(p, "contentChanges")));
        let Some(&last) = changes.last() else {
            return vec![];
        };
        let Some(text) = as_str(get(last, "text")).map(str::to_string) else {
            return vec![];
        };
        self.documents.insert(uri.clone(), text);
        vec![self.publish(&uri)]
    }

    /// Recompute diagnostics from the canonical parser and emit the push
    /// notification. No invented lints: only what parse() proves, which
    /// for a valid document is an empty list.
    fn publish(&self, uri: &str) -> String {
        let diagnostics = match self.documents.get(uri) {
            Some(text) => match analysis::analyze(text) {
                Ok(_) => vec![],
                Err(err) => vec![protocol::diagnostic(
                    text,
                    &err.message,
                    err.span.start,
                    err.span.end,
                )],
            },
            None => vec![],
        };
        publish_diagnostics(uri, &diagnostics)
    }

    fn document_symbols(&self, incoming: &protocol::Incoming) -> String {
        let Some((_uri, text)) = Self::uri_and_text(&incoming.params, &self.documents) else {
            return response(&incoming.id, Some("null".to_string()), None);
        };
        let Ok(analysis) = analysis::analyze(&text) else {
            // A broken document has no provable symbols; empty array, not
            // an error — diagnostics already reported why.
            return response(&incoming.id, Some("[]".to_string()), None);
        };
        let symbols: Vec<String> = analysis
            .defs
            .iter()
            .map(|def| {
                // SymbolKind 12 = Function; LSP has no Macro kind, so the
                // exact defining keyword travels in `detail`.
                format!(
                    "{{\"name\":{},\"detail\":{},\"kind\":12,\"range\":{},\"selectionRange\":{},\"children\":[]}}",
                    str_lit(&def.name),
                    str_lit(&def.kind),
                    span_to_range(&text, def.form_span.start, def.form_span.end),
                    span_to_range(&text, def.name_span.start, def.name_span.end),
                )
            })
            .collect();
        response(&incoming.id, Some(format!("[{}]", symbols.join(","))), None)
    }

    fn hover(&self, incoming: &protocol::Incoming) -> String {
        let Some((_, text)) = Self::uri_and_text(&incoming.params, &self.documents) else {
            return response(&incoming.id, Some("null".to_string()), None);
        };
        let offset = Self::cursor_offset(&incoming.params, &text);
        let Ok(analysis) = analysis::analyze(&text) else {
            return response(&incoming.id, Some("null".to_string()), None);
        };
        let Some((symbol, sym_span)) = analysis.symbol_at(&text, offset) else {
            return response(&incoming.id, Some("null".to_string()), None);
        };
        let Some(def) = analysis.lookup(&symbol) else {
            // Unknown stays unknown: hovering a built-in or undefined name
            // returns null rather than guessed documentation.
            return response(&incoming.id, Some("null".to_string()), None);
        };
        let value = format!(
            "**{}** `{}`\n\nDefined locally at bytes {}..{}\n\n```my-lisp\n{}\n```",
            def.kind,
            def.name,
            def.name_span.start,
            def.name_span.end,
            span_text(&text, def.form_span)
        );
        let result = format!(
            "{{\"contents\":{{\"kind\":\"markdown\",\"value\":{}}},\"range\":{}}}",
            str_lit(&value),
            span_to_range(&text, sym_span.start, sym_span.end)
        );
        response(&incoming.id, Some(result), None)
    }

    fn definition(&self, incoming: &protocol::Incoming) -> String {
        let Some((uri, text)) = Self::uri_and_text(&incoming.params, &self.documents) else {
            return response(&incoming.id, Some("null".to_string()), None);
        };
        let offset = Self::cursor_offset(&incoming.params, &text);
        let Ok(analysis) = analysis::analyze(&text) else {
            return response(&incoming.id, Some("null".to_string()), None);
        };
        let Some((symbol, _span)) = analysis.symbol_at(&text, offset) else {
            return response(&incoming.id, Some("null".to_string()), None);
        };
        let Some(def) = analysis.lookup(&symbol) else {
            return response(&incoming.id, Some("null".to_string()), None);
        };
        // M0 scope: same-document resolution only.
        let result = format!(
            "{{\"uri\":{},\"range\":{}}}",
            str_lit(&uri),
            span_to_range(&text, def.name_span.start, def.name_span.end)
        );
        response(&incoming.id, Some(result), None)
    }

    /// params.position.{line,character} → byte offset into the doc text.
    fn cursor_offset(params: &Option<Value>, text: &str) -> usize {
        let Some(position) = params.as_ref().and_then(|p| get(p, "position")) else {
            return usize::MAX;
        };
        let line = as_i64(get(position, "line")).unwrap_or(0).max(0) as u32;
        let character = as_i64(get(position, "character")).unwrap_or(0).max(0) as u32;
        let _ = offset_to_position; // mapping lives in analysis; used via position_to_offset below
        analysis::position_to_offset(text, line, character)
    }
}
