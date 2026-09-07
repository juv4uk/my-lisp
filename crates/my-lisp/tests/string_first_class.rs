use my_lisp::{eval_program, Session, Value};
use std::rc::Rc;

fn eval(source: &str) -> Value {
    eval_program(source, &mut Session::default())
        .unwrap_or_else(|error| panic!("evaluation failed: {error}\nsource: {source}"))
        .value
}

#[test]
fn string_append_is_a_first_class_value() {
    assert_eq!(
        eval("(def join string-append) (join \"left\" \"right\")"),
        Value::String(Rc::from("leftright")),
    );
}

#[test]
fn eager_string_builtin_can_be_passed_higher_order() {
    assert_eq!(
        eval("(def apply2 (lambda (f a b) (f a b))) (apply2 string-append \"a\" \"b\")"),
        Value::String(Rc::from("ab")),
    );
}

#[test]
fn eager_string_name_obeys_ordinary_lexical_shadowing() {
    assert_eq!(
        eval("((lambda (string-append) (string-append \"a\" \"b\")) (lambda (a b) (quote shadowed)))"),
        Value::Symbol(Rc::from("shadowed")),
    );
}

#[test]
fn all_migrated_string_mechanisms_keep_their_surface_behavior() {
    assert_eq!(eval("(string? \"x\")"), Value::Symbol(Rc::from("t")));
    assert_eq!(
        eval("(symbol->string (quote hello))"),
        Value::String(Rc::from("hello")),
    );
    assert_eq!(
        eval("(string->symbol \"hello\")"),
        Value::Symbol(Rc::from("hello")),
    );
    assert_eq!(
        eval("(string-first \"λisp\")"),
        Value::String(Rc::from("λ")),
    );
    assert_eq!(
        eval("(string-rest \"λisp\")"),
        Value::String(Rc::from("isp")),
    );
    assert_eq!(eval("(string<? \"a\" \"b\")"), Value::Symbol(Rc::from("t")));
}

#[test]
fn evaluator_source_does_not_dispatch_migrated_string_names() {
    let evaluator = include_str!("../src/eval/mod.rs");
    for name in [
        "string-append",
        "string<?",
        "string?",
        "symbol->string",
        "string->symbol",
        "string-first",
        "string-rest",
    ] {
        assert!(
            !evaluator.contains(&format!("Some(\"{name}\")")),
            "evaluator regained hard-coded ownership of {name}"
        );
    }
}
