#!/usr/bin/env python3
from pathlib import Path


def replace_exact(path: str, old: str, new: str, expected: int = 1) -> None:
    p = Path(path)
    text = p.read_text()
    count = text.count(old)
    if count != expected:
        raise SystemExit(f"{path}: expected {expected} occurrences, found {count}")
    p.write_text(text.replace(old, new))


# Necessary-form dispatch accepts machine identities as well as human peers.
replace_exact(
    "crates/my-lisp/src/eval/mod.rs",
    "necessary_forms::identity_for_surface(name)",
    "necessary_forms::identity_for_symbol(name)",
    expected=2,
)

Path("crates/my-lisp/src/eval/necessary_forms.rs").write_text(r'''//! Immutable routing for evaluator mechanisms necessary beyond Canon 0 + McCarthy7.
//!
//! Machine semantic authority lives in `lib/surface/semantic-registry.wsm`.
//! The Rust enum below selects an evaluator mechanism; it is NOT a semantic
//! identity registry. Each route is pinned to the numeric identity from the
//! authority file, and a unit test proves the small runtime spelling cache has
//! not drifted from that authority.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum NecessaryFormIdentity {
    Define,
    Lambda,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct NecessaryFormEntry {
    pub identity: NecessaryFormIdentity,
    pub semantic_id: &'static str,
    pub surfaces: &'static [&'static str],
}

pub(crate) const LAMBDA_SEMANTIC_ID: &str = "0010";
pub(crate) const DEFINE_SEMANTIC_ID: &str = "0011";

/// `#0010` / `#0011` are source-transport atoms only. The `#` is deliberately
/// not part of semantic identity: generated Lisp data uses the pure numeric
/// symbols `0010` / `0011`, created with the already-existing `string->symbol`.
/// Keeping transport in an ordinary atom avoids reserving new reader syntax.
fn transported_semantic_id(name: &str) -> &str {
    name.strip_prefix('#').unwrap_or(name)
}

/// Closed evaluator-routing cache. Numeric IDs are the machine handles; human
/// spellings are direct peers. CI proves these rows equal the stable rows in
/// the numeric registry, so this cache cannot silently become a second authority.
pub(crate) const NECESSARY_FORMS: [NecessaryFormEntry; 2] = [
    NecessaryFormEntry {
        identity: NecessaryFormIdentity::Define,
        semantic_id: DEFINE_SEMANTIC_ID,
        surfaces: &["define", "визначити"],
    },
    NecessaryFormEntry {
        identity: NecessaryFormIdentity::Lambda,
        semantic_id: LAMBDA_SEMANTIC_ID,
        surfaces: &["lambda", "функція"],
    },
];

/// Resolve an executable list-head symbol. A pure numeric semantic handle,
/// its collision-free source transport `#<id>`, and every ratified stable
/// human spelling enter the same evaluator mechanism directly.
pub(crate) fn identity_for_symbol(name: &str) -> Option<NecessaryFormIdentity> {
    let semantic_candidate = transported_semantic_id(name);
    NECESSARY_FORMS
        .iter()
        .find(|entry| {
            entry.semantic_id == semantic_candidate || entry.surfaces.contains(&name)
        })
        .map(|entry| entry.identity)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SEMANTIC_REGISTRY: &str =
        include_str!("../../../../lib/surface/semantic-registry.wsm");

    fn stable_registry_names(identity: &str) -> Vec<&str> {
        let prefix = format!("  ({identity} ");
        let line = SEMANTIC_REGISTRY
            .lines()
            .find(|line| line.starts_with(&prefix))
            .unwrap_or_else(|| panic!("semantic registry must contain identity {identity}"));
        let fields = line.split_whitespace().collect::<Vec<_>>();
        let (triples, remainder) = fields[1..].as_chunks::<3>();
        assert!(remainder.is_empty(), "malformed registry row: {line}");

        triples
            .iter()
            .filter_map(|triple| {
                let name = triple[1];
                let status = triple[2].trim_end_matches(')');
                (status == "stable" && name != "—").then_some(name)
            })
            .collect()
    }

    #[test]
    fn necessary_forms_have_two_numeric_semantic_identities() {
        assert_eq!(NECESSARY_FORMS.len(), 2);
        assert_eq!(
            identity_for_symbol(DEFINE_SEMANTIC_ID),
            Some(NecessaryFormIdentity::Define)
        );
        assert_eq!(
            identity_for_symbol(LAMBDA_SEMANTIC_ID),
            Some(NecessaryFormIdentity::Lambda)
        );
    }

    #[test]
    fn source_transport_routes_without_becoming_semantic_authority() {
        assert_eq!(
            identity_for_symbol("#0011"),
            Some(NecessaryFormIdentity::Define)
        );
        assert_eq!(
            identity_for_symbol("#0010"),
            Some(NecessaryFormIdentity::Lambda)
        );
        assert_eq!(identity_for_symbol("#define"), None);
        assert_eq!(identity_for_symbol("#9999"), None);
    }

    #[test]
    fn ukrainian_and_english_names_are_direct_peer_spellings() {
        assert_eq!(
            identity_for_symbol("визначити"),
            Some(NecessaryFormIdentity::Define)
        );
        assert_eq!(
            identity_for_symbol("define"),
            Some(NecessaryFormIdentity::Define)
        );
        assert_eq!(
            identity_for_symbol("функція"),
            Some(NecessaryFormIdentity::Lambda)
        );
        assert_eq!(
            identity_for_symbol("lambda"),
            Some(NecessaryFormIdentity::Lambda)
        );
    }

    #[test]
    fn runtime_routing_cache_matches_numeric_registry_authority() {
        for entry in NECESSARY_FORMS {
            let mut from_registry = stable_registry_names(entry.semantic_id);
            let mut cached = entry.surfaces.to_vec();
            from_registry.sort_unstable();
            cached.sort_unstable();
            assert_eq!(
                cached, from_registry,
                "necessary-form routing cache drifted from semantic identity {}",
                entry.semantic_id
            );
        }
    }

    #[test]
    fn compatibility_def_is_not_define_identity() {
        assert_eq!(identity_for_symbol("def"), None);
    }
}
''')

Path("lib/macro.my").write_text(r'''; lib/macro.my — macro definition derived inside my-lisp.
; lib/macro.my — визначення макросів, виведене всередині my-lisp.
;
; This file is the executable reduction proof for DEFMACRO:
;
;   DEFMACRO = DEFINE + LAMBDA + MAKE_MACRO + list construction
;
; `make-macro` is the narrow host substrate Closure -> Macro. The behavior of
; macro definition is still constructed here in the language itself.
;
; This source deliberately binds NO human surface name. It evaluates to one
; first-class Macro value; the bootstrap loader then exposes that same value
; directly under the ratified peer spellings `defmacro` and
; `визначити-макрос` (plus the historical `defmacro-derived` compatibility
; spelling). Thus no human surface is implemented as an alias of another.
;
; Necessary forms are selected by numeric semantic identity, not by an EN/UK
; spelling. `#0010` is only an ordinary-symbol source transport for bootstrap
; LAMBDA; `#` is not part of identity and requires no reader reservation.
; Expansion data itself contains the pure symbols `0010` / `0011`, constructed
; by the already-existing `string->symbol`. Sanskrit remains explicitly missing.

(make-macro
  (#0010 (name params . body)
    (cons (string->symbol "0011")
      (cons name
        (cons
          (cons (quote make-macro)
            (cons
              (cons (string->symbol "0010")
                (cons params body))
              (quote ())))
          (quote ()))))))
''')

Path("crates/my-lisp/tests/semantic_form_identity.rs").write_text(r'''use my_lisp::{eval_parsed_expressions, eval_program, parse, Expr, ExprKind, Session, Span};

const MACRO_LIBRARY: &str = include_str!("../../../lib/macro.my");

fn symbol(name: &str) -> Expr {
    Expr {
        kind: ExprKind::Symbol(name.into()),
        span: Span { start: 0, end: 0 },
    }
}

fn list(items: Vec<Expr>) -> Expr {
    Expr {
        kind: ExprKind::List(items.into()),
        span: Span { start: 0, end: 0 },
    }
}

fn walk_symbols(expression: &Expr, symbols: &mut Vec<String>) {
    match &expression.kind {
        ExprKind::Symbol(symbol) => symbols.push(symbol.to_string()),
        ExprKind::List(items) => {
            for item in items.iter() {
                walk_symbols(item, symbols);
            }
        }
        ExprKind::Pair(head, tail) => {
            walk_symbols(head, symbols);
            walk_symbols(tail, symbols);
        }
        _ => {}
    }
}

#[test]
fn existing_vertical_bar_atoms_remain_reader_compatible() {
    let parsed = parse("a|b").expect("vertical bar inside an atom must remain ordinary syntax");
    assert!(matches!(
        &parsed[0].kind,
        ExprKind::Symbol(value) if value.as_ref() == "a|b"
    ));
}

#[test]
fn source_transport_is_distinct_from_numeric_semantic_identity() {
    let mut session = Session::default();
    let result = eval_program(
        "(eq (string->symbol \"0010\") (quote #0010))",
        &mut session,
    )
    .expect("transport and identity comparison should evaluate");
    assert_eq!(result.value.to_string(), "()");
}

#[test]
fn source_transport_executes_define_and_lambda_without_reader_changes() {
    let mut session = Session::default();
    let result = eval_program(
        "(#0011 identity-by-transport (#0010 (x) x)) (identity-by-transport 42)",
        &mut session,
    )
    .expect("#0011/#0010 must route to DEFINE/LAMBDA mechanisms");
    assert_eq!(result.value.to_string(), "42");
}

#[test]
fn pure_numeric_symbols_execute_as_semantic_form_heads() {
    let definition = list(vec![
        symbol("0011"),
        symbol("identity-by-id"),
        list(vec![
            symbol("0010"),
            list(vec![symbol("x")]),
            symbol("x"),
        ]),
    ]);
    let mut forms = vec![definition];
    forms.extend(parse("(identity-by-id 43)").expect("call should parse"));

    let mut session = Session::default();
    let result = eval_parsed_expressions(&forms, &mut session)
        .expect("pure numeric semantic symbols must execute directly");
    assert_eq!(result.value.to_string(), "43");
}

#[test]
fn macro_library_executable_ast_contains_no_human_define_or_lambda_tokens() {
    let parsed = parse(MACRO_LIBRARY).expect("embedded macro library should parse");
    let mut symbols = Vec::new();
    for expression in &parsed {
        walk_symbols(expression, &mut symbols);
    }

    assert!(symbols.iter().any(|symbol| symbol == "#0010"));
    for forbidden in ["lambda", "функція", "define", "визначити"] {
        assert!(
            !symbols.iter().any(|symbol| symbol == forbidden),
            "macro.my must not select human necessary-form surface {forbidden}"
        );
    }
}

#[test]
fn defmacro_builds_and_runs_after_numeric_identity_lowering() {
    let mut session = Session::default();
    let result = eval_program(
        "(визначити-макрос identity-stage-e (x) x) (identity-stage-e 73)",
        &mut session,
    )
    .expect("direct defmacro peer should survive numeric-form lowering");
    assert_eq!(result.value.to_string(), "73");
}
''')
