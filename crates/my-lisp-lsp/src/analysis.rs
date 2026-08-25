//! analysis.rs — the language half of the adapter. Protocol-free by
//! design: everything here speaks in byte offsets, `Span`s and plain
//! structs, so no LSP shapes can leak into language reasoning and no
//! my-lisp semantics can leak into protocol glue.
//!
//! Everything is derived from the canonical parser (`my_lisp::parse`)
//! operating on the canonical source representation. There is no second
//! parser and no textual/grep-based definition detection: a definition
//! exists only where the parse tree structurally proves one — a top-level
//! `(def name ...)` or `(defmacro name ...)` list whose second element is
//! a symbol. Ordinary symbol text inside strings, comments or quoted data
//! is never classified as a definition because it never appears as a
//! top-level def-form in the AST.

use my_lisp::{Expr, ExprKind, LanguageError, Span};
use std::rc::Rc;

/// A definition the language can structurally prove.
#[derive(Clone, Debug)]
pub struct DefInfo {
    pub name: String,
    /// "def" or "defmacro" — exactly as written in the defining form.
    pub kind: String,
    /// Span of just the defined name (LSP selectionRange).
    pub name_span: Span,
    /// Span of the whole defining form (LSP range).
    pub form_span: Span,
}

/// One document's structural analysis, recomputed on every sync.
#[derive(Debug, Default)]
pub struct Analysis {
    pub defs: Vec<DefInfo>,
}

/// Parse with the canonical parser and collect what M0 needs.
/// A parse failure is returned untouched — inventing recovery here would
/// mean inventing semantics.
pub fn analyze(source: &str) -> Result<Analysis, LanguageError> {
    let expressions = my_lisp::parse(source)?;
    Ok(Analysis { defs: collect_defs(&expressions) })
}

fn collect_defs(expressions: &[Expr]) -> Vec<DefInfo> {
    let mut defs = Vec::new();
    for expr in expressions {
        let ExprKind::List(items) = &expr.kind else { continue };
        let Some(head) = items.first() else { continue };
        let ExprKind::Symbol(head_name) = &head.kind else { continue };
        if head_name.as_ref() != "def" && head_name.as_ref() != "defmacro" {
            continue;
        }
        // Structural proof requires the second element to be a symbol;
        // `(def (f a) ...)` or `(def "x" 1)` is not a provable named
        // definition, so it produces none rather than guessing one.
        let Some(second) = items.get(1) else { continue };
        let ExprKind::Symbol(name) = &second.kind else { continue };
        defs.push(DefInfo {
            name: name.to_string(),
            kind: head_name.to_string(),
            name_span: second.span,
            form_span: expr.span,
        });
    }
    defs
}

/// One occurrence of a symbol in source code.
#[derive(Clone, Debug)]
pub struct SymbolOccurrence {
    pub name: String,
    pub span: Span,
}

/// Collect every symbol occurrence that represents a *code reference*.
/// Subtrees under `(quote ...)` are data, not code, and are skipped —
/// this is structurally provable from the parse tree plus the fixed set
/// of special forms, so it stays inside the "nothing invented" boundary.
pub fn symbol_occurrences(source: &str) -> Result<Vec<SymbolOccurrence>, LanguageError> {
    let expressions = my_lisp::parse(source)?;
    let mut out = Vec::new();
    for expr in &expressions {
        walk_symbols(expr, false, &mut out);
    }
    Ok(out)
}

fn walk_symbols(expr: &Expr, in_quote: bool, out: &mut Vec<SymbolOccurrence>) {
    match &expr.kind {
        ExprKind::Symbol(name) => {
            if !in_quote {
                out.push(SymbolOccurrence { name: name.to_string(), span: expr.span });
            }
        }
        ExprKind::List(items) => {
            // `(quote data)` / `'data`: the whole subtree is data. The
            // reader-macro was removed in contract 2.0, so quote is the
            // only form to guard here.
            let head_is_quote = items
                .first()
                .map(|h| matches!(&h.kind, ExprKind::Symbol(n) if n.as_ref() == "quote"))
                .unwrap_or(false);
            for (i, item) in items.iter().enumerate() {
                walk_symbols(item, in_quote || (head_is_quote && i > 0), out);
            }
        }
        ExprKind::Pair(head, tail) => {
            walk_symbols(head, in_quote, out);
            walk_symbols(tail, in_quote, out);
        }
        _ => {}
    }
}

impl Analysis {
    /// Find the top-level expression containing `offset` and return its span.
    /// Used by hover to show evaluation results for complete forms.
    pub fn top_level_at(&self, source: &str, offset: usize) -> Option<Span> {
        let exprs = my_lisp::parse(source).ok()?;
        exprs.iter()
            .find(|e| e.span.start <= offset && offset < e.span.end)
            .map(|e| e.span)
    }

    pub fn lookup(&self, name: &str) -> Option<&DefInfo> {
        // Last definition wins, matching the evaluator's own shadowing of
        // a repeated `def` in one session/document.
        self.defs.iter().rev().find(|d| d.name == name)
    }

    /// The innermost symbol expression containing `offset`, found by
    /// walking the same parse tree — never by scanning raw text.
    pub fn symbol_at(&self, source: &str, offset: usize) -> Option<(String, Span)> {
        fn walk(expr: &Expr, offset: usize) -> Option<(String, Span)> {
            if !(expr.span.start <= offset && offset < expr.span.end) {
                return None;
            }
            match &expr.kind {
                ExprKind::Symbol(name) => Some((name.to_string(), expr.span)),
                ExprKind::List(items) => items.iter().find_map(|item| walk(item, offset)),
                ExprKind::Pair(head, tail) => {
                    walk(head, offset).or_else(|| walk(tail, offset))
                }
                _ => None,
            }
        }
        // Top-level forms come from parse(); if the offset falls between
        // forms there is simply nothing to find.
        my_lisp::parse(source).ok()?.iter().find_map(|e| walk(e, offset))
    }
}

// ---------------------------------------------------------------------------
// Position mapping. LSP positions are line / UTF-16 code units; the
// canonical spans are UTF-8 byte offsets. This conversion is pure adapter
// arithmetic and belongs nowhere near the core.
// ---------------------------------------------------------------------------

/// Byte offset → (line, character-in-UTF-16-units), end-exclusive spans
/// map naturally since LSP ranges are also end-exclusive. Offsets past the
/// end clamp to the end instead of failing — diagnostics must survive
/// truncated sources.
fn floor_char_boundary(source: &str, mut i: usize) -> usize {
    i = i.min(source.len());
    while i > 0 && !source.is_char_boundary(i) {
        i -= 1;
    }
    i
}

fn ceil_char_boundary(source: &str, mut i: usize) -> usize {
    i = i.min(source.len());
    while i < source.len() && !source.is_char_boundary(i) {
        i += 1;
    }
    i
}

pub fn offset_to_position(source: &str, offset: usize) -> (u32, u32) {
    let offset = floor_char_boundary(source, offset);
    let mut line = 0u32;
    let mut character = 0u32;
    for ch in source[..offset].chars() {
        if ch == '\n' {
            line += 1;
            character = 0;
        } else {
            character += ch.len_utf16() as u32;
        }
    }
    (line, character)
}

/// (line, character-in-UTF-16-units) → byte offset. Out-of-range positions
/// clamp to the nearest valid boundary for the same robustness reason.
pub fn position_to_offset(source: &str, line: u32, character: u32) -> usize {
    let mut current_line = 0u32;
    let mut line_start_byte = 0usize;
    if line > 0 {
        for (i, b) in source.bytes().enumerate() {
            if b == b'\n' {
                current_line += 1;
                line_start_byte = i + 1;
                if current_line == line {
                    break;
                }
            }
        }
        if current_line < line {
            return source.len();
        }
    }
    let rest = &source[line_start_byte..];
    let mut utf16_seen = 0u32;
    for (i, ch) in rest.char_indices() {
        if ch == '\n' || utf16_seen >= character {
            return line_start_byte + i;
        }
        utf16_seen += ch.len_utf16() as u32;
        if utf16_seen >= character {
            return line_start_byte + i + ch.len_utf8();
        }
    }
    source.len()
}

/// Render a span back to source text (for hover payload details).
pub fn span_text(source: &str, span: Span) -> &str {
    let start = floor_char_boundary(source, span.start);
    let end = ceil_char_boundary(source, span.end.max(start));
    &source[start..end]
}

// `Rc` is used by ExprKind; keep the import honest even if unused today.
#[allow(unused)]
fn _rc_witness(_: Rc<str>) {}

/// Static documentation for core builtins — used by hover when the symbol
/// is not locally defined. Only names that exist in the Rust evaluator
/// (eval/builtins.rs + special-forms dispatch) are listed here.
pub fn builtin_docs(name: &str) -> Option<&'static str> {
    match name {
        "+" => Some("Sum of all arguments"),
        "-" => Some("Difference (binary/subtract or unary/negate)"),
        "*" => Some("Product of all arguments"),
        "/" => Some("Exact rational division"),
        "<" => Some("Less-than chain comparison"),
        ">" => Some("Greater-than chain comparison"),
        "=" => Some("Numeric equality (value-based, not type-based)"),
        "atom" => Some("(atom x) → t if x is not a Pair"),
        "car" => Some("(car pair) → first element of a pair/list"),
        "cdr" => Some("(cdr pair) → rest of a pair/list after first"),
        "cons" => Some("(cons head tail) → new Pair(head, tail)"),
        "eq" => Some("(eq a b) → structural/identity equality"),
        "env" => Some("(env) → all visible bindings as alist"),
        "abs" => Some("(abs x) → absolute value (exact or inexact)"),
        "min" => Some("(min a b ...) → smallest argument"),
        "max" => Some("(max a b ...) → largest argument"),
        "min-list" => Some("(min-list lst) → minimum element of list"),
        "max-list" => Some("(max-list lst) → maximum element of list"),
        "make-vector" => Some("(make-vector n) → vector of n nil slots"),
        "vector" => Some("(vector a b ...) → vector with given elements"),
        "vector-length" => Some("(vector-length v) → number of elements"),
        "vector-ref" => Some("(vector-ref v i) → element at index i"),
        "vector-set!" => Some("(vector-set! v i val) → mutate slot i"),
        "print" => Some("(print x) → output x followed by newline"),
        "princ" => Some("(princ x) → output x without newline"),
        "read" => Some("(read) → read one s-expression from stdin"),
        "read-all" => Some("(read-all) → read all expressions from stdin"),
        "read-file" => Some("(read-file path) → file content as string (capability)"),
        "write-file" => Some("(write-file path data) → write to file (capability)"),
        "eval" => Some("(eval expr) → evaluate expression"),
        "quote" => Some("(quote x) → return x unevaluated"),
        "cond" => Some("(cond (test result)...) → first matching clause"),
        "lambda" => Some("(lambda (params) body) → anonymous function"),
        "def" => Some("(def name value) → bind name in current scope"),
        "defmacro" => Some("(defmacro name (params) body) → compile-time macro"),
        "json-parse" => Some("(json-parse str) → parse JSON to my-lisp values"),
        "sha256-hex" => Some("(sha256-hex str) → SHA-256 hex digest"),
        "string-append" => Some("(string-append a b ...) → concatenated strings"),
        "string<?" => Some("(string<? a b) → lexicographic less-than"),
        "string?" => Some("(string? x) → t if x is a string"),
        "symbol->string" => Some("(symbol->string sym) → string form of symbol"),
        "string->symbol" => Some("(string->symbol str) → symbol from string"),
        _ => None,
    }
}
