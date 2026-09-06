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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Arity {
    Exact(usize),
    AtLeast(usize),
    Between { min: usize, max: usize },
}

impl Arity {
    pub fn accepts(self, received: usize) -> bool {
        match self {
            Self::Exact(expected) => received == expected,
            Self::AtLeast(minimum) => received >= minimum,
            Self::Between { min, max } => (min..=max).contains(&received),
        }
    }

    pub fn expected(self) -> String {
        match self {
            Self::Exact(expected) => expected.to_string(),
            Self::AtLeast(minimum) => format!("at least {minimum}"),
            Self::Between { min, max } => format!("between {min} and {max}"),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LanguageItem {
    pub name: String,
    pub signature: &'static str,
    pub documentation: &'static str,
    pub kind: LanguageItemKind,
    pub arity: Arity,
}

const SYNTAX_FORMS: &[(&str, &str, &str, Arity)] = &[
    (
        "quote",
        "(quote value)",
        "Return value unevaluated",
        Arity::Exact(1),
    ),
    (
        "lambda",
        "(lambda (params) body ...)",
        "Create an anonymous function",
        Arity::AtLeast(2),
    ),
    (
        "def",
        "(def name value)",
        "Bind name in the current scope",
        Arity::Exact(2),
    ),
    (
        "defmacro",
        "(defmacro name (params) body ...)",
        "Bind a compile-time macro",
        Arity::AtLeast(3),
    ),
    (
        "cond",
        "(cond (test result) ...)",
        "Evaluate the first matching clause",
        Arity::AtLeast(0),
    ),
    (
        "print",
        "(print value)",
        "Output value followed by newline",
        Arity::Exact(1),
    ),
    (
        "princ",
        "(princ value)",
        "Output value without newline",
        Arity::Exact(1),
    ),
    (
        "write-to-string",
        "(write-to-string value)",
        "Return a readable string representation",
        Arity::Exact(1),
    ),
    (
        "read",
        "(read [source])",
        "Read one s-expression from a string or stdin",
        Arity::Between { min: 0, max: 1 },
    ),
    (
        "eval",
        "(eval expression)",
        "Evaluate an expression represented as data",
        Arity::Exact(1),
    ),
    (
        "string-append",
        "(string-append left right)",
        "Concatenate two strings",
        Arity::Exact(2),
    ),
    (
        "string<?",
        "(string<? left right)",
        "Compare strings lexicographically",
        Arity::Exact(2),
    ),
    (
        "read-all",
        "(read-all source)",
        "Read all expressions from a string",
        Arity::Exact(1),
    ),
    (
        "string?",
        "(string? value)",
        "Return t if value is a string",
        Arity::Exact(1),
    ),
    (
        "symbol->string",
        "(symbol->string symbol)",
        "Return the string form of a symbol",
        Arity::Exact(1),
    ),
    (
        "string->symbol",
        "(string->symbol string)",
        "Create a symbol from a string",
        Arity::Exact(1),
    ),
    (
        "string-first",
        "(string-first string)",
        "Return the first character as a string",
        Arity::Exact(1),
    ),
    (
        "string-rest",
        "(string-rest string)",
        "Return the string without its first character",
        Arity::Exact(1),
    ),
    (
        "sha256-hex",
        "(sha256-hex string)",
        "Return the SHA-256 hex digest",
        Arity::Exact(1),
    ),
    (
        "json-parse",
        "(json-parse string)",
        "Parse JSON into my-lisp values",
        Arity::Exact(1),
    ),
];

fn builtin_metadata(name: &str) -> (&'static str, &'static str, Arity) {
    match name {
        "+" => ("(+ number ...)", "Sum all arguments", Arity::AtLeast(0)),
        "-" => ("(- number ...)", "Subtract or negate", Arity::AtLeast(1)),
        "*" => (
            "(* number ...)",
            "Multiply all arguments",
            Arity::AtLeast(0),
        ),
        "/" => (
            "(/ number ...)",
            "Perform exact rational division",
            Arity::AtLeast(1),
        ),
        "<" => (
            "(< number ...)",
            "Less-than chain comparison",
            Arity::AtLeast(1),
        ),
        ">" => (
            "(> number ...)",
            "Greater-than chain comparison",
            Arity::AtLeast(1),
        ),
        "=" => ("(= number ...)", "Numeric equality", Arity::AtLeast(1)),
        "atom" => (
            "(atom value)",
            "Test whether value is not a pair",
            Arity::Exact(1),
        ),
        "car" => (
            "(car pair)",
            "Return the first element of a pair",
            Arity::Exact(1),
        ),
        "cdr" => ("(cdr pair)", "Return the tail of a pair", Arity::Exact(1)),
        "cons" => ("(cons head tail)", "Create a pair", Arity::Exact(2)),
        "eq" => (
            "(eq left right)",
            "Test structural or identity equality",
            Arity::Exact(2),
        ),
        "env" => (
            "(env)",
            "Return visible bindings as an alist",
            Arity::Exact(0),
        ),
        "abs" => ("(abs number)", "Return the absolute value", Arity::Exact(1)),
        "min" => (
            "(min number ...)",
            "Return the smallest argument",
            Arity::AtLeast(1),
        ),
        "max" => (
            "(max number ...)",
            "Return the largest argument",
            Arity::AtLeast(1),
        ),
        "min-list" => (
            "(min-list list)",
            "Return the smallest list element",
            Arity::Exact(1),
        ),
        "max-list" => (
            "(max-list list)",
            "Return the largest list element",
            Arity::Exact(1),
        ),
        "make-vector" => (
            "(make-vector length)",
            "Create a vector of nil slots",
            Arity::Exact(1),
        ),
        "vector" => (
            "(vector value ...)",
            "Create a vector containing the arguments",
            Arity::AtLeast(0),
        ),
        "vector-length" => (
            "(vector-length vector)",
            "Return the element count",
            Arity::Exact(1),
        ),
        "vector-ref" => (
            "(vector-ref vector index)",
            "Return an element by index",
            Arity::Exact(2),
        ),
        "vector-set!" => (
            "(vector-set! vector index value)",
            "Mutate a vector slot",
            Arity::Exact(3),
        ),
        "mono-ns" => (
            "(mono-ns)",
            "Return a monotonic nanosecond counter as an exact integer",
            Arity::Exact(0),
        ),
        "unix-time-now" => (
            "(unix-time-now)",
            "Observe the host wall clock as raw Unix seconds and nanoseconds",
            Arity::Exact(0),
        ),
        "ntp-query-raw" => (
            "(ntp-query-raw host timeout-ms)",
            "Perform one bounded NTP query and return raw protocol fields",
            Arity::Exact(2),
        ),
        "timezone-declarations-raw" => (
            "(timezone-declarations-raw)",
            "Observe raw TZ and /etc/timezone declaration candidates",
            Arity::Exact(0),
        ),
        "i32-buffer" => (
            "(i32-buffer number ...)",
            "Create a signed 32-bit numeric buffer",
            Arity::AtLeast(0),
        ),
        "f32-buffer" => (
            "(f32-buffer number ...)",
            "Create a binary32 numeric buffer",
            Arity::AtLeast(0),
        ),
        "numeric-buffer?" => (
            "(numeric-buffer? value)",
            "Test for a numeric buffer",
            Arity::Exact(1),
        ),
        "numeric-buffer-type" => (
            "(numeric-buffer-type buffer)",
            "Return i32 or f32",
            Arity::Exact(1),
        ),
        "numeric-buffer-length" => (
            "(numeric-buffer-length buffer)",
            "Return the element count",
            Arity::Exact(1),
        ),
        "numeric-buffer-ref" => (
            "(numeric-buffer-ref buffer index)",
            "Return an element by index",
            Arity::Exact(2),
        ),
        "numeric-buffer-map" => (
            "(numeric-buffer-map function buffer)",
            "Map a function over a numeric buffer",
            Arity::Exact(2),
        ),
        "string-slice" => (
            "(string-slice string start end)",
            "Return a UTF-8-safe substring",
            Arity::Exact(3),
        ),
        _ => (
            "(builtin ...)",
            "First-class runtime builtin",
            Arity::AtLeast(0),
        ),
    }
}

pub fn language_items() -> Vec<LanguageItem> {
    let mut items = Environment::root()
        .snapshot()
        .into_iter()
        .filter_map(|(name, value)| {
            matches!(value, Value::Builtin(_)).then(|| {
                let (signature, documentation, arity) = builtin_metadata(&name);
                LanguageItem {
                    name: name.to_string(),
                    signature,
                    documentation,
                    kind: LanguageItemKind::Builtin,
                    arity,
                }
            })
        })
        .collect::<Vec<_>>();

    items.extend(
        SYNTAX_FORMS
            .iter()
            .map(|(name, signature, documentation, arity)| LanguageItem {
                name: (*name).to_string(),
                signature,
                documentation,
                kind: LanguageItemKind::SyntaxForm,
                arity: *arity,
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
                let matches = items
                    .iter()
                    .filter(|item| {
                        item.kind == LanguageItemKind::Builtin
                            && item.name.as_str() == name.as_ref()
                    })
                    .collect::<Vec<_>>();
                assert_eq!(
                    matches.len(),
                    1,
                    "runtime builtin {name} must have exactly one tooling item"
                );
                assert_ne!(
                    matches[0].signature, "(builtin ...)",
                    "runtime builtin {name} needs an explicit signature"
                );
                assert_ne!(
                    matches[0].documentation, "First-class runtime builtin",
                    "runtime builtin {name} needs explicit documentation"
                );
            }
        }
    }
}
