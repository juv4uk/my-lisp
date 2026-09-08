use my_lisp::{eval_parsed_expressions, eval_program, parse, Expr, ExprKind, Session, Span};

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
