//! Discoverable language items for tooling.
//!
//! Runtime builtin names come from the same root environment used by the
//! evaluator. Tooling therefore cannot silently retain a stale copy when a
//! first-class builtin is added. Syntax-dispatched forms remain explicit
//! because they are not ordinary environment bindings.

use crate::{Environment, Value};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LanguageItemKind {
    Builtin,
    SyntaxForm,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LanguageItem {
    pub name: String,
    pub documentation: &'static str,
    pub kind: LanguageItemKind,
}

const SYNTAX_FORMS: &[(&str, &str)] = &[
    ("quote", "(quote x) → return x unevaluated"),
    ("lambda", "(lambda (params) body) → anonymous function"),
    ("def", "(def name value) → bind name in current scope"),
    (
        "defmacro",
        "(defmacro name (params) body) → compile-time macro",
    ),
    ("cond", "(cond (test result)...) → first matching clause"),
    ("print", "(print x) → output x followed by newline"),
    ("princ", "(princ x) → output x without newline"),
    (
        "write-to-string",
        "(write-to-string x) → readable string representation",
    ),
    ("read", "(read) → read one s-expression from stdin"),
    (
        "eval",
        "(eval expr) → evaluate an expression represented as data",
    ),
    (
        "string-append",
        "(string-append a b ...) → concatenated strings",
    ),
    ("string<?", "(string<? a b) → lexicographic less-than"),
    ("read-all", "(read-all) → read all expressions from stdin"),
    ("string?", "(string? x) → t if x is a string"),
    (
        "symbol->string",
        "(symbol->string sym) → string form of symbol",
    ),
    (
        "string->symbol",
        "(string->symbol str) → symbol from string",
    ),
    (
        "string-first",
        "(string-first str) → first character as a string",
    ),
    (
        "string-rest",
        "(string-rest str) → string without its first character",
    ),
    ("sha256-hex", "(sha256-hex str) → SHA-256 hex digest"),
    (
        "json-parse",
        "(json-parse str) → parse JSON to my-lisp values",
    ),
];

fn builtin_documentation(name: &str) -> &'static str {
    match name {
        "+" => "Sum of all arguments",
        "-" => "Difference (binary subtraction or unary negation)",
        "*" => "Product of all arguments",
        "/" => "Exact rational division",
        "<" => "Less-than chain comparison",
        ">" => "Greater-than chain comparison",
        "=" => "Numeric equality (value-based, not type-based)",
        "atom" => "(atom x) → t if x is not a pair",
        "car" => "(car pair) → first element of a pair/list",
        "cdr" => "(cdr pair) → rest of a pair/list after first",
        "cons" => "(cons head tail) → new pair",
        "eq" => "(eq a b) → structural/identity equality",
        "env" => "(env) → all visible bindings as an alist",
        "abs" => "(abs x) → absolute value",
        "min" => "(min a b ...) → smallest argument",
        "max" => "(max a b ...) → largest argument",
        "min-list" => "(min-list list) → smallest list element",
        "max-list" => "(max-list list) → largest list element",
        "make-vector" => "(make-vector n) → vector of n nil slots",
        "vector" => "(vector a b ...) → vector containing the arguments",
        "vector-length" => "(vector-length v) → number of vector elements",
        "vector-ref" => "(vector-ref v i) → vector element at index i",
        "vector-set!" => "(vector-set! v i value) → mutate vector slot i",
        "mono-ms" => "(mono-ms) → exact monotonic milliseconds",
        "mono-ns" => "(mono-ns) → exact monotonic nanoseconds",
        "i32-buffer" => "(i32-buffer n ...) → signed 32-bit numeric buffer",
        "f32-buffer" => "(f32-buffer n ...) → binary32 numeric buffer",
        "numeric-buffer?" => "(numeric-buffer? x) → t for a numeric buffer",
        "numeric-buffer-type" => "(numeric-buffer-type b) → i32 or f32",
        "numeric-buffer-length" => "(numeric-buffer-length b) → element count",
        "numeric-buffer-ref" => "(numeric-buffer-ref b i) → element at index i",
        "numeric-buffer-map" => "(numeric-buffer-map f b) → mapped numeric buffer",
        "string-slice" => "(string-slice str start end) → UTF-8-safe substring",
        _ => "First-class runtime builtin",
    }
}

pub fn language_items() -> Vec<LanguageItem> {
    let mut items = Environment::root()
        .snapshot()
        .into_iter()
        .filter_map(|(name, value)| {
            matches!(value, Value::Builtin(_)).then(|| LanguageItem {
                documentation: builtin_documentation(&name),
                name: name.to_string(),
                kind: LanguageItemKind::Builtin,
            })
        })
        .collect::<Vec<_>>();

    items.extend(
        SYNTAX_FORMS
            .iter()
            .map(|(name, documentation)| LanguageItem {
                name: (*name).to_string(),
                documentation,
                kind: LanguageItemKind::SyntaxForm,
            }),
    );
    items
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_root_builtin_is_discoverable_exactly_once() {
        let items = language_items();
        for (name, value) in Environment::root().snapshot() {
            if matches!(value, Value::Builtin(_)) {
                assert_eq!(
                    items
                        .iter()
                        .filter(|item| {
                            item.kind == LanguageItemKind::Builtin
                                && item.name.as_str() == name.as_ref()
                        })
                        .count(),
                    1,
                    "runtime builtin {name} must have exactly one tooling item"
                );
            }
        }
    }
}
