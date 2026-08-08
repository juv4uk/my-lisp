use my_lisp::{eval_program, parse, ErrorKind, Rational, Session, Value};

fn eval(source: &str) -> Value {
    eval_program(source, &mut Session::default()).unwrap().value
}

#[test]
fn division_is_an_exact_reduced_rational() {
    assert_eq!(
        eval("(/ 5 6 8 7)"),
        Value::Rational(Rational::new(5, 336).unwrap())
    );
    assert_eq!(eval("(/ 8 4)"), Value::Number(2.0));
    assert_eq!(
        eval("(/ (/ 2 3))"),
        Value::Rational(Rational::new(3, 2).unwrap())
    );
}

/// `Rational` used to be `i64`-bounded and this exact expression overflowed
/// (`ErrorKind::InvalidForm`) — deliberately kept *out* of
/// tests/fixtures/conformance.json at the time (see that file's README)
/// because whether a future bignum-capable implementation should still
/// overflow here was an open scope question, not yet a decided contract.
/// `crates/my-lisp/src/bignum.rs` answered it: `Rational` is now backed by
/// a hand-rolled arbitrary-precision integer (no crate dependency — see
/// its header comment for why), so this now computes the exact product
/// instead of erroring. Kept as a Rust-only regression test, still not
/// promoted to the shared contract, since a future C or HDL implementation
/// might reasonably choose a different (or still bounded) representation.
#[test]
fn exact_arithmetic_handles_products_beyond_i64_range() {
    let result = eval_program("(* 3037000500 3037000500)", &mut Session::default()).unwrap();
    assert_eq!(result.value.to_string(), "9223372037000250000");
}

/// The case that actually matters, more than any single large literal:
/// results *computed* via repeated exact arithmetic growing past the old
/// i64 ceiling. `(/ 1 1)` forces the exact path from the start (a bare
/// integer literal this large would itself parse as inexact f64 — see
/// docs/language-core.md — a separate, still-open question from whether
/// *arithmetic* stays exact past i64, which this answers: yes). Verified
/// against Python's `math.factorial(30)` by hand before writing this.
#[test]
fn exact_arithmetic_computes_factorials_past_i64_range() {
    let source = r#"
        (def fact
          (lambda (n acc)
            (cond
              ((eq n 0) acc)
              (t (fact (- n 1) (* acc n))))))
        (fact 30 (/ 1 1))
    "#;
    let result = eval_program(source, &mut Session::default()).unwrap();
    assert_eq!(result.value.to_string(), "265252859812191058636308480000000");
}

#[test]
fn arithmetic_promotes_exact_integers_and_preserves_inexact_numbers() {
    assert_eq!(
        eval("(+ (/ 1 3) (/ 1 3))"),
        Value::Rational(Rational::new(2, 3).unwrap())
    );
    assert_eq!(
        eval("(- 1 (/ 1 3))"),
        Value::Rational(Rational::new(2, 3).unwrap())
    );
    assert_eq!(
        eval("(* (/ 2 3) (/ 9 4))"),
        Value::Rational(Rational::new(3, 2).unwrap())
    );
    assert_eq!(
        eval("(- (/ 1 3))"),
        Value::Rational(Rational::new(-1, 3).unwrap())
    );
    assert_eq!(eval("(+ (/ 1 2) 0.25)"), Value::Number(0.75));
    assert_eq!(eval("(+ (/ 1 2) (/ 1 2))"), Value::Number(1.0));
}

#[test]
fn comparisons_chain_and_promote_exact_inexact_like_arithmetic() {
    assert_eq!(eval("(< 1 2 3)"), Value::Bool(true));
    assert_eq!(eval("(< 1 3 2)"), Value::Bool(false));
    assert_eq!(eval("(> 3 2 1)"), Value::Bool(true));
    assert_eq!(eval("(> 3 1 2)"), Value::Bool(false));
    assert_eq!(eval("(= 1 1 1)"), Value::Bool(true));
    assert_eq!(eval("(= 1 2)"), Value::Bool(false));
    // One inexact operand makes the whole comparison inexact, same rule as +/-/*.
    assert_eq!(eval("(= 1 1.0)"), Value::Bool(true));
    // Cross-multiplication compares exact fractions without ever going through f64.
    assert_eq!(eval("(= 1/2 0.5)"), Value::Bool(true));
    assert_eq!(eval("(< (/ 1 3) (/ 1 2))"), Value::Bool(true));
    // A single argument is vacuously ordered/equal.
    assert_eq!(eval("(< 5)"), Value::Bool(true));
    assert_eq!(eval("(<= 1 1 2)"), Value::Bool(true));
    assert_eq!(eval("(<= 1 2 1)"), Value::Bool(false));
    assert_eq!(eval("(>= 3 3 2)"), Value::Bool(true));
    assert_eq!(eval("(>= 2 3)"), Value::Bool(false));
    assert_eq!(eval("(<= 1/2 0.5)"), Value::Bool(true));
}

#[test]
fn comparison_with_no_arguments_is_an_arity_error() {
    let error = eval_program("(<)", &mut Session::default()).unwrap_err();
    assert_eq!(error.kind, ErrorKind::Arity);
}

#[test]
fn print_appends_to_output_and_returns_its_argument() {
    let result = eval_program("(print \"radio\")", &mut Session::default()).unwrap();
    assert_eq!(result.value, Value::String("radio".into()));
    assert_eq!(result.output, vec!["\"radio\"".to_string()]);
}

#[test]
fn print_composes_inside_expressions_and_accumulates_in_order() {
    let result = eval_program("(+ (print 1) (print 2))", &mut Session::default()).unwrap();
    assert_eq!(result.value, Value::Number(3.0));
    assert_eq!(result.output, vec!["1".to_string(), "2".to_string()]);
}

#[test]
fn read_parses_text_into_data_without_evaluating_it() {
    assert_eq!(
        eval(r#"(read "(+ 1 2)")"#),
        Value::list([
            Value::Symbol("+".into()),
            Value::Number(1.0),
            Value::Number(2.0),
        ])
    );
    assert_eq!(eval(r#"(read "radio")"#), Value::Symbol("radio".into()));
    assert_eq!(eval(r#"(read "42")"#), Value::Number(42.0));
}

#[test]
fn read_rejects_non_string_arguments_and_multi_expression_input() {
    let non_string = eval_program("(read 42)", &mut Session::default()).unwrap_err();
    assert_eq!(non_string.kind, ErrorKind::Type);

    let two_expressions = eval_program(r#"(read "1 2")"#, &mut Session::default()).unwrap_err();
    assert_eq!(two_expressions.kind, ErrorKind::InvalidForm);

    let too_many_args = eval_program(r#"(read "1" "2")"#, &mut Session::default()).unwrap_err();
    assert_eq!(too_many_args.kind, ErrorKind::Arity);
}

#[test]
fn eval_closes_the_read_eval_loop_by_hand() {
    assert_eq!(eval(r#"(eval (read "(+ 1 2)"))"#), Value::Number(3.0));
    assert_eq!(eval("(eval (quote (+ 1 2)))"), Value::Number(3.0));
}

#[test]
fn eval_looks_up_a_quoted_symbol_in_the_calling_environment() {
    let mut session = Session::default();
    eval_program("(def x 5)", &mut session).unwrap();
    let result = eval_program("(eval 'x)", &mut session).unwrap();
    assert_eq!(result.value, Value::Number(5.0));
}

#[test]
fn eval_treats_closures_and_macros_as_self_evaluating() {
    let mut session = Session::default();
    let closure = eval_program("(eval (lambda (x) x))", &mut session).unwrap();
    assert!(matches!(closure.value, Value::Closure(_)));
}

#[test]
fn print_inside_a_closure_shares_the_root_sessions_output() {
    // Environment::child() must share the parent's output sink (not start a
    // fresh one per call frame), or `print` inside a lambda body would be
    // invisible to the caller's EvalResult.output.
    let source = "((lambda () (print 'inside) 'done))";
    let result = eval_program(source, &mut Session::default()).unwrap();
    assert_eq!(result.value, Value::Symbol("done".into()));
    assert_eq!(result.output, vec!["inside".to_string()]);
}

#[test]
fn tail_recursion_uses_constant_rust_stack() {
    let depth = 5_000;
    let mut definitions = (0..depth - 1)
        .map(|index| format!("(def step-{index} (lambda () (step-{})))", index + 1))
        .collect::<Vec<_>>();
    definitions.push(format!("(def step-{} (lambda () 'done))", depth - 1));
    let source = format!("{} (step-0)", definitions.join(" "));
    assert_eq!(eval(&source), Value::Symbol("done".into()));
}

#[test]
fn bootstrap_library_is_written_and_executed_in_my_lisp() {
    let mut session = Session::default();
    eval_program(include_str!("../../../lib/core.my"), &mut session).unwrap();
    assert_eq!(
        eval_program("(second '(radio antenna))", &mut session)
            .unwrap()
            .value,
        Value::Symbol("antenna".into())
    );
    assert_eq!(
        eval_program("(not '())", &mut session).unwrap().value,
        Value::Bool(true)
    );
}

#[test]
fn bootstrap_library_provides_list_utilities() {
    let mut session = Session::default();
    eval_program(include_str!("../../../lib/core.my"), &mut session).unwrap();
    let run = |source: &str, session: &mut Session| {
        eval_program(source, session).unwrap().value.to_string()
    };
    assert_eq!(run("(length '(radio antenna signal))", &mut session), "3");
    assert_eq!(run("(length '())", &mut session), "0");
    assert_eq!(run("(reverse '(1 2 3))", &mut session), "(3 2 1)");
    assert_eq!(run("(append '(1 2) '(3 4))", &mut session), "(1 2 3 4)");
    assert_eq!(
        run("(map (lambda (x) (+ x 1)) '(1 2 3))", &mut session),
        "(2 3 4)"
    );
    assert_eq!(
        run("(filter (lambda (x) (eq x 2)) '(1 2 3 2))", &mut session),
        "(2 2)"
    );
    assert_eq!(
        run("(reduce (lambda (acc x) (+ acc x)) 0 '(1 2 3 4))", &mut session),
        "10"
    );
}

#[test]
fn bootstrap_library_provides_let_and_let_star() {
    let mut session = Session::default();
    eval_program(include_str!("../../../lib/core.my"), &mut session).unwrap();
    let run = |source: &str, session: &mut Session| {
        eval_program(source, session).unwrap().value.to_string()
    };
    assert_eq!(run("(let ((x 1) (y 2)) (+ x y))", &mut session), "3");
    assert_eq!(run("(let () 42)", &mut session), "42");
    // Parallel, not sequential: y's value expression can't see x yet.
    let parallel_shadowing_fails =
        eval_program("(let ((x 1) (y x)) (+ x y))", &mut session).unwrap_err();
    assert_eq!(parallel_shadowing_fails.kind, ErrorKind::UnknownSymbol);
    // A let binding shadows an outer def without mutating it.
    assert_eq!(run("(def z 100) (let ((z 1)) z)", &mut session), "1");
    assert_eq!(run("z", &mut session), "100");
    // let* threads each binding's value through to the ones after it.
    assert_eq!(
        run(
            "(let* ((x 1) (y (+ x 1)) (z (+ y 1))) (list x y z))",
            &mut session
        ),
        "(1 2 3)"
    );
    assert_eq!(run("(let* () 7)", &mut session), "7");
}

#[test]
fn bootstrap_library_provides_deep_structural_equality() {
    let mut session = Session::default();
    eval_program(include_str!("../../../lib/core.my"), &mut session).unwrap();
    let run = |source: &str, session: &mut Session| {
        eval_program(source, session).unwrap().value.to_string()
    };
    assert_eq!(run("(equal? '(1 2 3) '(1 2 3))", &mut session), "t");
    assert_eq!(run("(equal? '(1 2 3) '(1 2 4))", &mut session), "()");
    assert_eq!(
        run("(equal? '(1 (2 3) 4) '(1 (2 3) 4))", &mut session),
        "t"
    );
    assert_eq!(run("(equal? '() '())", &mut session), "t");
    assert_eq!(run("(equal? 'radio 'radio)", &mut session), "t");
    // Different lengths, and an atom compared against a compound term —
    // neither should ever reach `eq` with a non-atom operand.
    assert_eq!(run("(equal? '(1 2) '(1 2 3))", &mut session), "()");
    assert_eq!(run("(equal? 5 '(5))", &mut session), "()");
    assert_eq!(run("(equal? '(1 2) 5)", &mut session), "()");
}

#[test]
fn reader_supports_unicode_comments_and_quote_sugar() {
    let expressions = parse("; коментар\n'радіо").unwrap();
    assert_eq!(expressions.len(), 1);
    assert_eq!(eval("'радіо"), Value::Symbol("радіо".into()));
}

#[test]
fn implements_mccarthys_seven_primitives() {
    assert_eq!(eval("(quote radio)"), Value::Symbol("radio".into()));
    assert_eq!(eval("(atom 'radio)"), Value::Bool(true));
    assert_eq!(eval("(atom '())"), Value::Bool(true));
    assert_eq!(eval("(atom '(radio antenna))"), Value::Bool(false));
    assert_eq!(eval("(eq 'radio 'radio)"), Value::Bool(true));
    assert_eq!(eval("(eq 'radio 'antenna)"), Value::Bool(false));
    assert_eq!(
        eval("(car '(radio antenna))"),
        Value::Symbol("radio".into())
    );
    assert_eq!(
        eval("(cdr '(radio antenna))"),
        Value::list([Value::Symbol("antenna".into())])
    );
    assert_eq!(
        eval("(cons 'radio '(antenna))"),
        Value::list([
            Value::Symbol("radio".into()),
            Value::Symbol("antenna".into())
        ])
    );
    assert_eq!(
        eval("(cond (() 'wrong) (t 'right))"),
        Value::Symbol("right".into())
    );
}

#[test]
fn reports_structured_errors_with_source_spans() {
    let error = eval_program("(car '())", &mut Session::default()).unwrap_err();
    assert_eq!(error.kind, ErrorKind::Type);
    assert_eq!((error.span.start, error.span.end), (0, 9));

    let parse_error = parse("(cons 'a").unwrap_err();
    assert_eq!(parse_error.kind, ErrorKind::Parse);
    assert_eq!(parse_error.span.start, 0);
}

#[test]
fn lexical_child_reads_parent_without_mutating_it() {
    let parent = my_lisp::Environment::root();
    let child = parent.child();
    child.define("station", Value::Symbol("UR5ABC".into()));
    assert_eq!(child.get("t"), Some(Value::Bool(true)));
    assert_eq!(parent.get("station"), None);
}

#[test]
fn lambda_captures_lexical_environment_and_keeps_parameters_local() {
    let mut session = Session::default();
    session
        .environment
        .define("station", Value::Symbol("radio".into()));

    let result = eval_program(
        "((lambda (suffix) (cons station suffix)) '(antenna))",
        &mut session,
    )
    .unwrap();

    assert_eq!(
        result.value,
        Value::list([
            Value::Symbol("radio".into()),
            Value::Symbol("antenna".into())
        ])
    );
    assert_eq!(session.environment.get("suffix"), None);
}

#[test]
fn lambda_is_a_first_class_value() {
    assert_eq!(
        eval("((lambda (apply-once) (apply-once 'radio)) (lambda (x) (cons x '())))"),
        Value::list([Value::Symbol("radio".into())])
    );
}

#[test]
fn lambda_reports_invalid_parameters_and_arity() {
    let duplicate = eval_program("(lambda (x x) x)", &mut Session::default()).unwrap_err();
    assert_eq!(duplicate.kind, ErrorKind::InvalidForm);
    assert!(duplicate.message.contains("повторний параметр"));

    let invalid = eval_program("(lambda (1) 1)", &mut Session::default()).unwrap_err();
    assert_eq!(invalid.kind, ErrorKind::InvalidForm);

    let arity = eval_program("((lambda (x) x))", &mut Session::default()).unwrap_err();
    assert_eq!(arity.kind, ErrorKind::Arity);
}

/// tests/fixtures/conformance.json is the implementation-independent contract
/// (see CLAUDE.md): any future my-lisp implementation — C, HDL, whatever —
/// should reproduce these results once it gets the seven primitives and
/// lambda/def/defmacro right, since everything above that (lib/core.my
/// included) is plain my-lisp source, not Rust. Preloading core.my here lets
/// fixtures exercise it directly instead of duplicating stdlib coverage.
/// tests/fixtures/conformance.json — незалежний від реалізації контракт
/// (див. CLAUDE.md): будь-яка майбутня реалізація my-lisp — C, HDL, що
/// завгодно — має відтворювати ці результати, щойно правильно реалізує сім
/// примітивів і lambda/def/defmacro, бо все, що над ними (включно з
/// lib/core.my), — звичайний my-lisp-код, не Rust. Попереднє завантаження
/// core.my тут дозволяє фікстурам напряму його використовувати замість
/// дублювання покриття stdlib.
/// tests/fixtures/conformance.json ist der implementierungsunabhängige
/// Vertrag (siehe CLAUDE.md): jede künftige my-lisp-Implementierung — C,
/// HDL, was auch immer — sollte diese Ergebnisse reproduzieren, sobald sie
/// die sieben Primitive und lambda/def/defmacro korrekt umsetzt, da alles
/// darüber (inklusive lib/core.my) gewöhnlicher my-lisp-Quellcode ist, kein
/// Rust. Das Vorladen von core.my erlaubt es Fixtures, es direkt zu nutzen,
/// statt Stdlib-Abdeckung zu duplizieren.
#[test]
fn conformance_tests_from_json() {
    use my_lisp_literate::{eval_literate, SourceMode};
    use serde_json::Value as Json;

    let json: Json =
        serde_json::from_str(include_str!("../../../tests/fixtures/conformance.json"))
            .expect("conformance.json should be valid JSON");
    let fixtures = json.as_array().expect("conformance.json should be an array");

    let mut session = Session::default();
    eval_program(include_str!("../../../lib/core.my"), &mut session)
        .expect("lib/core.my should load before conformance fixtures run");
    eval_program(include_str!("../../../lib/unify.my"), &mut session)
        .expect("lib/unify.my should load before conformance fixtures run");
    eval_program(include_str!("../../../lib/reason.my"), &mut session)
        .expect("lib/reason.my should load before conformance fixtures run");

    for fixture in fixtures {
        let expr = fixture["expr"].as_str().expect("fixture needs an \"expr\" string");
        let is_markdown = fixture.get("mode").and_then(Json::as_str) == Some("markdown");

        if let Some(expected_error) = fixture.get("error").and_then(Json::as_str) {
            assert!(
                !is_markdown,
                "error fixtures aren't supported in markdown mode: {expr}"
            );
            let error = eval_program(expr, &mut session)
                .expect_err(&format!("expected an error but evaluation succeeded: {expr}"));
            assert_eq!(
                format!("{:?}", error.kind),
                expected_error,
                "wrong error kind for expression: {expr}"
            );
            continue;
        }

        let expected = fixture["expected"]
            .as_str()
            .expect("fixture needs an \"expected\" string (or an \"error\" string)");

        let actual = if is_markdown {
            eval_literate(expr, SourceMode::Literate, &mut session)
                .unwrap_or_else(|e| panic!("markdown fixture failed: {e}\nexpr: {expr}"))
                .0
                .value
                .to_string()
        } else {
            eval_program(expr, &mut session)
                .unwrap_or_else(|e| panic!("fixture failed: {e}\nexpr: {expr}"))
                .value
                .to_string()
        };
        assert_eq!(actual, expected, "Failed on expression: {}", expr);
    }
}

// Minimal symbol/string introspection this project held off on for a long
// time (CLAUDE.md: don't grow the Rust surface) — added deliberately when
// lib/clips-import.my's Step 2 needed to strip CLIPS's `?` prefix off a
// variable symbol, which is impossible from within my-lisp itself without
// some way to look at a symbol's characters.

#[test]
fn symbol_to_string_and_back_round_trips() {
    assert_eq!(eval("(symbol->string 'planet)").to_string(), "\"planet\"");
    assert_eq!(
        eval("(string->symbol (symbol->string 'planet))").to_string(),
        "planet"
    );
}

#[test]
fn string_first_returns_a_one_character_string() {
    assert_eq!(
        eval("(string-first (symbol->string '?x))").to_string(),
        "\"?\""
    );
}

#[test]
fn string_rest_drops_exactly_the_first_character() {
    assert_eq!(
        eval("(string-rest (symbol->string '?x))").to_string(),
        "\"x\""
    );
}

#[test]
fn symbol_to_string_rejects_a_non_symbol() {
    let error = eval_program("(symbol->string \"already a string\")", &mut Session::default())
        .expect_err("expected a Type error");
    assert_eq!(error.kind, ErrorKind::Type);
}

#[test]
fn string_rest_rejects_an_empty_string() {
    let error = eval_program(
        r#"(string-rest (symbol->string (string->symbol "")))"#,
        &mut Session::default(),
    )
    .expect_err("expected a Type error on an empty string");
    assert_eq!(error.kind, ErrorKind::Type);
}
