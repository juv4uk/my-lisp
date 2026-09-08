use my_lisp::{
    eval_parsed_expressions, eval_program, parse, ErrorKind, Expr, ExprKind, Session, Span,
};

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
fn first_class_eval_can_bootstrap_lambda_from_pure_numeric_identity() {
    let mut session = Session::default();
    let result = eval_program(
        "((eval (cons (string->symbol \"0010\") (quote ((x) x)))) 41)",
        &mut session,
    )
    .expect("eval must turn a pure 0010-headed datum into a closure");
    assert_eq!(result.value.to_string(), "41");
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
fn macro_library_selects_no_human_or_transport_spelling_for_necessary_forms() {
    let parsed = parse(MACRO_LIBRARY).expect("embedded macro library should parse");
    let mut symbols = Vec::new();
    for expression in &parsed {
        walk_symbols(expression, &mut symbols);
    }

    assert!(MACRO_LIBRARY.contains("\"0010\""));
    assert!(MACRO_LIBRARY.contains("\"0011\""));
    for forbidden in ["lambda", "функція", "define", "визначити", "#0010", "#0011"] {
        assert!(
            !symbols.iter().any(|symbol| symbol == forbidden),
            "macro.my must not select necessary-form spelling {forbidden}"
        );
    }
}

#[test]
fn defmacro_preserves_minimum_arity_failure_class() {
    for source in ["(defmacro)", "(defmacro only-a-name)"] {
        let error = eval_program(source, &mut Session::default())
            .expect_err("defmacro below its two-argument minimum must fail named");
        assert_eq!(error.kind, ErrorKind::Arity, "source: {source}");
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
