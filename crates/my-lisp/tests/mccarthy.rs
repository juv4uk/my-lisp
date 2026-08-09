use my_lisp::{eval_program, parse, Environment, ErrorKind, Exactness, Expr, ExprKind, Rational, Session, Value};

/// Looks up `key` in a my-lisp alist `((k1 . v1) (k2 . v2) ...)`, already
/// parsed as `Expr`s (data, not evaluated) — used by the two
/// `tests/fixtures/conformance.my`-consuming tests below, which read the
/// fixture file as reader-level data rather than executing it.
fn alist_str<'a>(entries: &'a [Expr], key: &str) -> Option<&'a str> {
    entries.iter().find_map(|entry| {
        let ExprKind::Pair(k, v) = &entry.kind else {
            return None;
        };
        let ExprKind::Symbol(name) = &k.kind else {
            return None;
        };
        if &**name != key {
            return None;
        }
        match &v.kind {
            ExprKind::String(s) => Some(s.as_ref()),
            _ => None,
        }
    })
}

/// Same as `alist_str`, but for a numeric field (e.g. `tier`).
fn alist_number(entries: &[Expr], key: &str) -> Option<f64> {
    entries.iter().find_map(|entry| {
        let ExprKind::Pair(k, v) = &entry.kind else {
            return None;
        };
        let ExprKind::Symbol(name) = &k.kind else {
            return None;
        };
        if &**name != key {
            return None;
        }
        match &v.kind {
            ExprKind::Number(n, _) => Some(*n),
            _ => None,
        }
    })
}

fn eval(source: &str) -> Value {
    eval_program(source, &mut Session::default()).unwrap().value
}

#[test]
fn division_is_an_exact_reduced_rational() {
    assert_eq!(
        eval("(/ 5 6 8 7)"),
        Value::Rational(Rational::new(5, 336).unwrap())
    );
    assert_eq!(eval("(/ 8 4)"), Value::Number(2.0, Exactness::Exact));
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
    assert_eq!(eval("(+ (/ 1 2) 0.25)"), Value::Number(0.75, Exactness::Inexact));
    assert_eq!(eval("(+ (/ 1 2) (/ 1 2))"), Value::Number(1.0, Exactness::Exact));
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
    assert_eq!(result.value, Value::Number(3.0, Exactness::Exact));
    assert_eq!(result.output, vec!["1".to_string(), "2".to_string()]);
}

#[test]
fn read_parses_text_into_data_without_evaluating_it() {
    assert_eq!(
        eval(r#"(read "(+ 1 2)")"#),
        Value::list([
            Value::Symbol("+".into()),
            Value::Number(1.0, Exactness::Exact),
            Value::Number(2.0, Exactness::Exact),
        ])
    );
    assert_eq!(eval(r#"(read "radio")"#), Value::Symbol("radio".into()));
    assert_eq!(eval(r#"(read "42")"#), Value::Number(42.0, Exactness::Exact));
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
    assert_eq!(eval(r#"(eval (read "(+ 1 2)"))"#), Value::Number(3.0, Exactness::Exact));
    assert_eq!(eval("(eval (quote (+ 1 2)))"), Value::Number(3.0, Exactness::Exact));
}

#[test]
fn eval_looks_up_a_quoted_symbol_in_the_calling_environment() {
    let mut session = Session::default();
    eval_program("(def x 5)", &mut session).unwrap();
    let result = eval_program("(eval 'x)", &mut session).unwrap();
    assert_eq!(result.value, Value::Number(5.0, Exactness::Exact));
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

/// Variadic parameters (2026-08-09, PLAN.md item 8's follow-on): three
/// shapes shared across the Lisp family, not one dialect's `&rest`
/// keyword — `(a b . rest)` (dotted list, reusing the same reader support
/// added earlier for data literals), a bare symbol (zero fixed params,
/// every argument), and the existing `(a b)` (exact arity, unchanged).
/// Варіативні параметри (2026-08-09, продовження пункту 8 з PLAN.md): три
/// форми, спільні для родини Lisp, не ключове слово `&rest` одного
/// діалекту — `(a b . rest)` (dotted-список, та сама підтримка reader'а,
/// додана раніше для літералів даних), голий символ (нуль фіксованих
/// параметрів, кожен аргумент), і наявний `(a b)` (точна арність, без змін).
#[test]
fn dotted_lambda_list_binds_extra_arguments_as_a_rest_list() {
    assert_eq!(
        eval("((lambda (a b . rest) rest) 1 2 3 4 5)"),
        Value::list(vec![Value::Number(3.0, Exactness::Exact), Value::Number(4.0, Exactness::Exact), Value::Number(5.0, Exactness::Exact)])
    );
    assert_eq!(eval("((lambda (a . rest) a) 1 2 3)"), Value::Number(1.0, Exactness::Exact));
}

#[test]
fn bare_symbol_lambda_list_binds_every_argument_as_one_list() {
    assert_eq!(
        eval("((lambda args args) 1 2 3)"),
        Value::list(vec![Value::Number(1.0, Exactness::Exact), Value::Number(2.0, Exactness::Exact), Value::Number(3.0, Exactness::Exact)])
    );
    assert_eq!(eval("((lambda args args))"), Value::Nil);
}

#[test]
fn variadic_lambda_still_requires_its_fixed_parameters() {
    let error = eval_program("((lambda (a b . rest) a) 1)", &mut Session::default()).unwrap_err();
    assert_eq!(error.kind, ErrorKind::Arity);
    assert!(error.message.contains("at least"));
}

#[test]
fn variadic_defmacro_binds_unevaluated_rest_arguments() {
    let mut session = Session::default();
    let result = eval_program(
        "(defmacro my-list items (cons 'quote (cons items '()))) (my-list 1 2 3)",
        &mut session,
    )
    .unwrap();
    assert_eq!(
        result.value,
        Value::list(vec![Value::Number(1.0, Exactness::Exact), Value::Number(2.0, Exactness::Exact), Value::Number(3.0, Exactness::Exact)])
    );
}

/// `Display`/`print` previously wrote `"{value}"` with no escaping at all —
/// a string containing a literal `"` broke `read ∘ print = identity`
/// silently (the printed text wasn't valid to read back: it would close
/// early on the embedded quote). Found 2026-08-09 while building tooling
/// that prints fixture data containing real quotes. Fixed by giving
/// `print` real `prin1`/`write` semantics (Common Lisp/Scheme's own
/// convention for the "read-back-safe" print function): escape `"`, `\`,
/// `\n`, `\t`.
/// `Display`/`print` раніше писали `"{value}"` без жодного екранування —
/// рядок з буквальною `"` мовчки ламав `read ∘ print = identity`
/// (надрукований текст не читався назад коректно: закривався зарано на
/// вбудованій лапці). Знайдено 2026-08-09 під час написання тулінгу, що
/// друкує дані фікстур із реальними лапками. Виправлено наданням `print`
/// справжньої семантики `prin1`/`write` (власна конвенція Common
/// Lisp/Scheme для "безпечної для read" функції друку): екранувати `"`,
/// `\`, `\n`, `\t`.
#[test]
fn print_escapes_embedded_quotes_and_backslashes_so_read_can_reconstruct_the_string() {
    // A string value containing a literal " and \, built via my-lisp source
    // escaping — the *value* itself is `(eq "radio" "radio")`, 22 chars,
    // no backslashes in the value, just in how it's written here.
    let source = r#""(eq \"radio\" \"radio\")""#;
    let value = eval_program(source, &mut Session::default())
        .unwrap()
        .value;
    // `to_string()` is now valid my-lisp source for that same string literal
    // — parsing it again (not wrapping in another layer of quoting) should
    // reconstruct the identical value.
    let printed = value.to_string();
    let reread = eval_program(&printed, &mut Session::default())
        .unwrap()
        .value;
    assert_eq!(reread, value, "printed text should read back to the same string value");
}

/// `princ` — the `princ`/`display` half of the classic Lisp print-function
/// pair `print` (fixed above) is the other half of: raw text, no quotes or
/// escapes, for output meant for a person or reassembled as literal source
/// text (e.g. a tool generating new .my files), never re-parsed as data.
/// `princ` — «princ»/«display»-половина класичної Lisp-пари функцій друку,
/// другу половину якої складає полагоджений вище `print`: сирий текст, без
/// лапок і екранування, для виводу, призначеного людині чи повторному
/// складанню як буквальний сирцевий текст (напр. інструмент, що генерує
/// новий `.my`-файл), ніколи не для повторного парсингу як даних.
#[test]
fn princ_outputs_a_string_raw_without_quotes_or_escapes() {
    let mut session = Session::default();
    let result = eval_program(r#"(princ "(eq \"radio\" \"radio\")")"#, &mut session).unwrap();
    assert_eq!(result.output, vec![r#"(eq "radio" "radio")"#.to_string()]);
    // princ still returns the string value itself, just like print does —
    // composes the same way, only the transcript text differs.
    assert_eq!(
        result.value,
        Value::String(r#"(eq "radio" "radio")"#.into())
    );
}

#[test]
fn princ_and_print_render_symbols_and_numbers_identically() {
    assert_eq!(
        eval_program("(princ 'radio)", &mut Session::default())
            .unwrap()
            .output,
        vec!["radio".to_string()]
    );
    assert_eq!(
        eval_program("(princ 42)", &mut Session::default())
            .unwrap()
            .output,
        vec!["42".to_string()]
    );
}

/// `list` used to be a Rust special form; moved to `lib/core.my` the same
/// day variadic lambda parameters were added, since `(def list (lambda
/// args args))` expresses it exactly — G4/G5's own filter ("can the
/// existing core already say this?") applied to the Rust surface itself,
/// not just to `.my` code.
/// `list` раніше був спеціальною формою Rust; перенесено в `lib/core.my`
/// того самого дня, коли додано варіативні параметри lambda, бо `(def list
/// (lambda args args))` виражає це точно — той самий фільтр G4/G5 ("чи
/// наявне ядро вже може це сказати?"), застосований до самого Rust-шару,
/// не лише до `.my`-коду.
#[test]
fn list_is_a_my_lisp_function_in_core_my_not_a_rust_builtin() {
    let mut session = Session::default();
    eval_program(include_str!("../../../lib/core.my"), &mut session).unwrap();
    let result = eval_program("(list 1 2 3)", &mut session).unwrap();
    assert_eq!(
        result.value,
        Value::list(vec![Value::Number(1.0, Exactness::Exact), Value::Number(2.0, Exactness::Exact), Value::Number(3.0, Exactness::Exact)])
    );
    // Without core.my loaded, "list" is an ordinary unbound symbol now —
    // regression-tests that it really did leave the Rust special-form table.
    let unbound = eval_program("(list 1 2 3)", &mut Session::default()).unwrap_err();
    assert_eq!(unbound.kind, ErrorKind::UnknownSymbol);
}

/// tests/fixtures/conformance.my is the implementation-independent contract
/// (see CLAUDE.md): any future my-lisp implementation — C, HDL, whatever —
/// should reproduce these results once it gets the seven primitives and
/// lambda/def/defmacro right, since everything above that (lib/core.my
/// included) is plain my-lisp source, not Rust. Preloading core.my here lets
/// fixtures exercise it directly instead of duplicating stdlib coverage.
/// Written as my-lisp data (2026-08-09, moved off JSON), so this test reads
/// it via `parse` — the same reader every my-lisp program goes through —
/// not `serde_json`; the fixture file no longer needs a foreign format to
/// stay implementation-independent, it needs my-lisp's own reader, which
/// every conforming implementation already has by definition.
/// tests/fixtures/conformance.my — незалежний від реалізації контракт
/// (див. CLAUDE.md): будь-яка майбутня реалізація my-lisp — C, HDL, що
/// завгодно — має відтворювати ці результати, щойно правильно реалізує сім
/// примітивів і lambda/def/defmacro, бо все, що над ними (включно з
/// lib/core.my), — звичайний my-lisp-код, не Rust. Попереднє завантаження
/// core.my тут дозволяє фікстурам напряму його використовувати замість
/// дублювання покриття stdlib. Записано як my-lisp-дані (2026-08-09,
/// перенесено з JSON), тож цей тест читає файл через `parse` — той самий
/// reader, крізь який проходить будь-яка my-lisp-програма — не через
/// `serde_json`; файлу фікстур більше не потрібен чужий формат, щоб
/// лишатись незалежним від реалізації, йому потрібен власний reader
/// my-lisp, який будь-яка конформна реалізація вже має за визначенням.
#[test]
fn conformance_tests_from_my() {
    let forms = parse(include_str!("../../../tests/fixtures/conformance.my"))
        .expect("conformance.my should parse as valid my-lisp source");

    let mut session = Session::default();
    eval_program(include_str!("../../../lib/core.my"), &mut session)
        .expect("lib/core.my should load before conformance fixtures run");
    eval_program(include_str!("../../../lib/unify.my"), &mut session)
        .expect("lib/unify.my should load before conformance fixtures run");
    eval_program(include_str!("../../../lib/reason.my"), &mut session)
        .expect("lib/reason.my should load before conformance fixtures run");
    eval_program(include_str!("../../../lib/understand.my"), &mut session)
        .expect("lib/understand.my should load before conformance fixtures run");
    eval_program(include_str!("../../../lib/narrate.my"), &mut session)
        .expect("lib/narrate.my should load before conformance fixtures run");

    for form in &forms {
        let ExprKind::List(entries) = &form.kind else {
            panic!("each top-level form in conformance.my should be an alist: {form:?}");
        };
        let expr = alist_str(entries, "expr").expect("fixture needs an \"expr\" string");

        if let Some(expected_error) = alist_str(entries, "error") {
            let error = eval_program(expr, &mut session)
                .expect_err(&format!("expected an error but evaluation succeeded: {expr}"));
            assert_eq!(
                format!("{:?}", error.kind),
                expected_error,
                "wrong error kind for expression: {expr}"
            );
            continue;
        }

        let expected =
            alist_str(entries, "expected").expect("fixture needs an \"expected\" string (or an \"error\" string)");
        let actual = eval_program(expr, &mut session)
            .unwrap_or_else(|e| panic!("fixture failed: {e}\nexpr: {expr}"))
            .value
            .to_string();
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
fn read_all_parses_every_top_level_form_as_data() {
    // Unlike `read`, which errors unless the string holds exactly one
    // form, `read-all` returns every top-level form as a list of data.
    assert_eq!(
        eval("(read-all \"(a b) (c d) 5\")").to_string(),
        "((a b) (c d) 5)"
    );
}

#[test]
fn read_all_rejects_a_non_string() {
    let error = eval_program("(read-all '(a b))", &mut Session::default())
        .expect_err("expected a Type error");
    assert_eq!(error.kind, ErrorKind::Type);
}

#[test]
fn string_predicate_distinguishes_strings_from_other_atoms() {
    assert_eq!(eval("(string? \"hello\")").to_string(), "t");
    assert_eq!(eval("(string? 'hello)").to_string(), "()");
    assert_eq!(eval("(string? 5)").to_string(), "()");
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

// --- dotted pairs: read ∘ print must be identity ------------------------
// Before this, `'(p . 0)` read as a *proper* 3-element list containing the
// literal symbol `.` in the middle — not a real dotted pair — even though
// the printer renders a genuine `(cons 'p 0)` with exactly that same text.
// The two structures printed identically but were never `equal?`. This is
// exactly the P2 axiom violation flagged while discussing
// `my-lisp-constitution.json`: every value must round-trip through
// read/print as itself, and a printed dotted pair must read back as one.

// `equal?` lives in lib/core.my, not the primitive core `eval()` above
// preloads — these two need it, so they load core.my themselves.
fn eval_with_core(source: &str) -> Value {
    let mut session = Session::default();
    eval_program(include_str!("../../../lib/core.my"), &mut session).unwrap();
    eval_program(source, &mut session).unwrap().value
}

#[test]
fn a_quoted_dotted_pair_literal_equals_the_cons_it_prints_as() {
    assert_eq!(
        eval_with_core("(equal? '(p . 0) (cons 'p 0))").to_string(),
        "t"
    );
}

#[test]
fn read_of_a_printed_dotted_pair_reconstructs_the_same_structure() {
    // The literal round-trip: `(cons 'p 0)` prints as the text "(p . 0)"
    // (see value.rs's `write_pair`); feeding that exact text back through
    // `read` must reconstruct something `equal?` to the original cons cell.
    assert_eq!(
        eval_with_core(r#"(equal? (read "(p . 0)") (cons 'p 0))"#).to_string(),
        "t"
    );
}

#[test]
fn a_multi_element_dotted_list_reads_as_nested_pairs() {
    assert_eq!(eval("'(a b . c)").to_string(), "(a b . c)");
    assert_eq!(eval("(car '(a b . c))").to_string(), "a");
    assert_eq!(eval("(car (cdr '(a b . c)))").to_string(), "b");
    assert_eq!(eval("(cdr (cdr '(a b . c)))").to_string(), "c");
}

#[test]
fn a_dotted_pair_used_directly_as_code_is_an_invalid_form() {
    // Only meaningful as data (inside `quote`, or via `read`) — a dotted
    // pair is not a valid call form, the same way `(1 2 3)` isn't.
    let error = eval_program("(p . 0)", &mut Session::default())
        .expect_err("expected an InvalidForm error");
    assert_eq!(error.kind, ErrorKind::InvalidForm);
}

/// `my-lisp-constitution.my` is a *generated projection* over
/// `tests/fixtures/conformance.my` (`scripts/build-constitution.my`
/// regenerates it) — the same one-source-plus-projection shape
/// `lib/knowledge.my`'s `*knowledge-journal*` uses for runtime state,
/// applied here to documentation instead. This test is the CI-enforced
/// half of that pattern: if someone appends a fixture to `conformance.my`
/// and forgets to rerun the generator, the two files silently drift — this
/// test turns that into a loud, immediate failure instead. Both files are
/// my-lisp data now (2026-08-09, moved off JSON), so this test parses them
/// the same way `conformance_tests_from_my` does above, not via serde_json.
/// `my-lisp-constitution.my` — це *згенерована проекція* над
/// `tests/fixtures/conformance.my` (перегенеровує `scripts/build-constitution.my`)
/// — та сама форма "одне джерело + проекція", яку `*knowledge-journal*`
/// з `lib/knowledge.my` використовує для рантайм-стану, застосована тут до
/// документації. Цей тест — примусова CI-половина того патерну: якщо хтось
/// додасть фікстуру в `conformance.my` і забуде перегенерувати, ці два
/// файли мовчки розійдуться — цей тест перетворює це на негайний, гучний
/// провал. Обидва файли тепер my-lisp-дані (2026-08-09, перенесено з JSON),
/// тож цей тест парсить їх так само, як `conformance_tests_from_my` вище,
/// не через `serde_json`.
#[test]
fn constitution_my_stays_in_sync_with_conformance_my() {
    let conformance = parse(include_str!("../../../tests/fixtures/conformance.my"))
        .expect("conformance.my should parse as valid my-lisp source");

    let constitution_forms = parse(include_str!("../../../my-lisp-constitution.my"))
        .expect("my-lisp-constitution.my should parse as valid my-lisp source");
    let fixtures: Vec<&[Expr]> = constitution_forms
        .iter()
        .filter_map(|form| {
            let ExprKind::List(items) = &form.kind else {
                return None;
            };
            // `(print (cons 'fixture fixture))` in build-constitution.my
            // prints as `(fixture (expr . ...) (expected . ...) ...)` — the
            // fixture alist's own entries spliced in as `cons`'s tail, not
            // wrapped in a nested list, since `fixture` here is already a
            // proper list and `(a . (b c))` prints flat as `(a b c)`.
            let (head, entries) = items.split_first()?;
            let ExprKind::Symbol(name) = &head.kind else {
                return None;
            };
            if &**name != "fixture" {
                return None;
            }
            Some(entries)
        })
        .collect();

    assert_eq!(
        conformance.len(),
        fixtures.len(),
        "my-lisp-constitution.my has a different fixture count than conformance.my — \
         run `cargo run -p my-lisp-cli -- scripts/build-constitution.my > my-lisp-constitution.my` to regenerate it"
    );

    for (i, (fact_form, tagged_entries)) in conformance.iter().zip(fixtures.iter()).enumerate() {
        let ExprKind::List(fact_entries) = &fact_form.kind else {
            panic!("conformance.my fixture #{} should be an alist", i + 1);
        };
        for key in ["expr", "expected", "error"] {
            assert_eq!(
                alist_str(fact_entries, key),
                alist_str(tagged_entries, key),
                "fixture #{} field \"{key}\" drifted between conformance.my and \
                 my-lisp-constitution.my — regenerate it",
                i + 1
            );
        }
    }
}

/// Project principle 3 ("build the reasoning machine") deliberately has no
/// G/S axiom counterpart in `docs/language-core-axioms.md` — an axiom is a
/// claim about the language, this principle is a claim about why the
/// project exists, and those are different categories on purpose. But that
/// leaves nothing in the language contract itself that would notice if
/// `lib/unify.my`/`lib/reason.my` were quietly deleted, or Tier 3 coverage
/// thinned out over time — the erosion would only be caught by whoever
/// happened to remember to look. This test is a process guard, not a
/// semantic one: it doesn't test what `unify`/`reason` mean (that's
/// `tests/unify.rs`/`tests/reason.rs`), only that they still exist, still
/// load, still prove one real fact, and that Tier 3 hasn't silently shrunk
/// below a floor. If the floor is intentionally being lowered, lower this
/// assertion explicitly — don't let it drift unnoticed.
/// Принцип проєкту 3 ("реалізувати розумну машину") свідомо не має
/// відповідника серед G/S аксіом у `docs/language-core-axioms.md` — аксіома
/// це твердження про мову, цей принцип — твердження про те, чому проєкт
/// існує, і це різні категорії навмисно. Але це означає, що ніщо в самому
/// мовному контракті не помітить, якщо `lib/unify.my`/`lib/reason.my` тихо
/// видалять, або покриття Рівня 3 з часом зменшиться — ерозію впіймає лише
/// той, хто випадково згадає подивитись. Цей тест — процесна гарантія, не
/// семантична: він не перевіряє, що означають `unify`/`reason` (це роблять
/// `tests/unify.rs`/`tests/reason.rs`), лише що вони й досі існують,
/// завантажуються, доводять один реальний факт, і що Рівень 3 мовчки не
/// просів нижче межі. Якщо межу свідомо знижують — знизити цю перевірку
/// явно, не дати їй розмитись непоміченою.
#[test]
fn symbolic_reasoning_layer_stays_loaded_and_tested() {
    let mut session = Session::default();
    eval_program(include_str!("../../../lib/core.my"), &mut session)
        .expect("lib/core.my should load before the symbolic layer");
    eval_program(include_str!("../../../lib/unify.my"), &mut session)
        .expect("lib/unify.my should load — the symbolic reasoning layer must stay present");
    eval_program(include_str!("../../../lib/reason.my"), &mut session)
        .expect("lib/reason.my should load — the symbolic reasoning layer must stay present");

    let result = eval_program(
        "(let ((rules '(((parent alice bob))))) (reason '(parent alice bob) rules))",
        &mut session,
    )
    .expect("reason should still actually prove a fact, not just load without error");
    assert_eq!(
        result.value.to_string(),
        "((() (proved (parent alice bob) (parent alice bob) ())))"
    );

    let forms = parse(include_str!("../../../tests/fixtures/conformance.my"))
        .expect("conformance.my should parse as valid my-lisp source");
    let tier3_count = forms
        .iter()
        .filter(|form| {
            let ExprKind::List(entries) = &form.kind else {
                return false;
            };
            alist_number(entries, "tier") == Some(3.0)
        })
        .count();
    assert!(
        tier3_count >= 20,
        "Tier 3 (ECOSYSTEM CONFORMANCE, which includes unify/reason) fixture count dropped to \
         {tier3_count} — project principle 3 names symbolic reasoning a project goal, not an \
         optional add-on; if this floor is intentionally being lowered, lower this assertion \
         explicitly instead of letting coverage drift down unnoticed"
    );
}

/// S3 named `OutOfMemory` in its own prose before the category existed in
/// code (found during the 2026-08-09 pre-ratification axiom audit) — this
/// makes it real: an opt-in cons-cell cap, simulating a genuinely bounded
/// heap (S3's own example, "4096 cons cells on an FPGA") without needing
/// real hardware to verify the claim "bounded implementations fail named,
/// never silently redefine `cons`'s meaning." The default session (every
/// `conformance.my` fixture) stays unbounded — this is opt-in, not a new
/// default limit on the reference implementation.
/// S3 назвав `OutOfMemory` у власному тексті до того, як категорія
/// існувала в коді (знайдено під час аудиту аксіом перед ратифікацією,
/// 2026-08-09) — цей тест робить її реальною: опційна межа на кількість
/// cons-комірок, що імітує справді обмежену купу (власний приклад S3,
/// "4096 cons-комірок на FPGA") без потреби в реальному залізі, щоб
/// перевірити твердження "обмежені реалізації провалюються названо,
/// ніколи не переозначають сенс `cons` мовчки". Типова сесія (кожна
/// фікстура `conformance.my`) лишається необмеженою — це опційно, не нова
/// типова межа для еталонної реалізації.
#[test]
fn cons_respects_an_opt_in_resource_limit_and_fails_named_not_silently() {
    let mut session = Session {
        environment: Environment::root().with_cons_limit(2),
    };
    eval_program("(cons 1 2)", &mut session).expect("first cons should succeed");
    eval_program("(cons 3 4)", &mut session).expect("second cons should succeed");
    let error =
        eval_program("(cons 5 6)", &mut session).expect_err("third cons should hit the limit");
    assert_eq!(error.kind, ErrorKind::OutOfMemory);
}

#[test]
fn cons_stays_unbounded_by_default_matching_every_conformance_fixture() {
    // The default Session::default() (what conformance_tests_from_my uses)
    // never opts into a limit — confirms OutOfMemory is reachable only when
    // a session deliberately asks for it, not a new default restriction.
    let mut session = Session::default();
    for _ in 0..10_000 {
        eval_program("(cons 1 2)", &mut session).expect("unbounded session should never run out");
    }
}

/// Same shape as the `cons` limit above, for `S1`'s own named example
/// (`NumericOverflow`) instead of `S3`'s (`OutOfMemory`) — an opt-in
/// bit-length cap on exact arithmetic results. Never falls back to an
/// inexact approximation past the limit (that would violate S1, not
/// satisfy it) — it fails named instead.
/// Та сама форма, що й межа `cons` вище, для власного названого прикладу
/// `S1` (`NumericOverflow`) замість `S3` (`OutOfMemory`) — опційна межа в
/// бітах на результати точної арифметики. Ніколи не відкочується до
/// неточного наближення за межею (це порушило б S1, не задовольнило б
/// його) — натомість провалюється названо.
#[test]
fn arithmetic_respects_an_opt_in_numeric_bit_limit_and_fails_named_not_silently() {
    let mut session = Session {
        environment: Environment::root().with_numeric_bit_limit(8), // fits up to 255
    };
    eval_program("(+ 100 100)", &mut session).expect("200 fits in 8 bits");
    let error = eval_program("(+ 200 200)", &mut session)
        .expect_err("400 exceeds an 8-bit limit and must not silently approximate");
    assert_eq!(error.kind, ErrorKind::NumericOverflow);
}

#[test]
fn division_respects_the_same_opt_in_numeric_bit_limit() {
    let mut session = Session {
        environment: Environment::root().with_numeric_bit_limit(8),
    };
    let error = eval_program("(/ 1 1000)", &mut session)
        .expect_err("a denominator past the bit limit must fail named");
    assert_eq!(error.kind, ErrorKind::NumericOverflow);
}

#[test]
fn arithmetic_stays_unbounded_by_default_matching_every_conformance_fixture() {
    let mut session = Session::default();
    eval_program(
        "(def big (lambda (n acc) (cond ((eq n 0) acc) (t (big (- n 1) (* acc 2)))))) (big 100 1)",
        &mut session,
    )
    .expect("unbounded session should compute a 100-bit result without a limit error");
}

// --- write-file (PLAN.md item 13) ---------------------------------------
// The write-side counterpart to read-file: always creates or
// truncates-and-overwrites the target, never appends. Uses a real temp
// file (std::env::temp_dir(), no crate dependency — this crate stays
// zero-dependency by design) rather than a fixture, since it's a real
// filesystem side effect, not a pure expression.

#[test]
fn write_file_then_read_file_round_trips_the_same_content() {
    let path = std::env::temp_dir().join("my-lisp-write-file-round-trip.txt");
    // Forward slashes only: my-lisp's string reader treats an unrecognized
    // backslash escape as "drop the backslash, keep the character" (only
    // \n/\t/\"/\\ are special — see parser.rs's `string` method), so a raw
    // Windows path like `C:\Users\...` embedded in a double-quoted literal
    // would silently lose every backslash instead of erroring.
    let path_str = path.to_str().expect("temp path should be valid UTF-8").replace('\\', "/");
    let source = format!(r#"(write-file "{path_str}" "hello from my-lisp")"#);
    let mut session = Session::default();
    let result = eval_program(&source, &mut session).expect("write-file should succeed");
    // write-file returns its content argument unchanged, like print does with its value.
    assert_eq!(result.value, Value::String("hello from my-lisp".into()));

    let read_back = eval_program(&format!(r#"(read-file "{path_str}")"#), &mut session)
        .expect("read-file should read back what write-file wrote");
    assert_eq!(read_back.value, Value::String("hello from my-lisp".into()));

    std::fs::remove_file(&path).ok();
}

#[test]
fn write_file_overwrites_rather_than_appends() {
    let path = std::env::temp_dir().join("my-lisp-write-file-overwrite.txt");
    // Forward slashes only: my-lisp's string reader treats an unrecognized
    // backslash escape as "drop the backslash, keep the character" (only
    // \n/\t/\"/\\ are special — see parser.rs's `string` method), so a raw
    // Windows path like `C:\Users\...` embedded in a double-quoted literal
    // would silently lose every backslash instead of erroring.
    let path_str = path.to_str().expect("temp path should be valid UTF-8").replace('\\', "/");
    let mut session = Session::default();
    eval_program(&format!(r#"(write-file "{path_str}" "first")"#), &mut session)
        .expect("first write-file should succeed");
    eval_program(&format!(r#"(write-file "{path_str}" "second")"#), &mut session)
        .expect("second write-file should succeed");
    let read_back = eval_program(&format!(r#"(read-file "{path_str}")"#), &mut session)
        .expect("read-file should see only the second write");
    assert_eq!(read_back.value, Value::String("second".into()));

    std::fs::remove_file(&path).ok();
}

#[test]
fn write_file_rejects_a_non_string_path() {
    let error = eval_program(r#"(write-file 42 "x")"#, &mut Session::default())
        .expect_err("a non-string path must fail named, not panic");
    assert_eq!(error.kind, ErrorKind::Type);
}

#[test]
fn write_file_rejects_a_non_string_content_argument() {
    let error = eval_program(r#"(write-file "path-does-not-matter-here.txt" 42)"#, &mut Session::default())
        .expect_err("a non-string content argument must fail named, not panic");
    assert_eq!(error.kind, ErrorKind::Type);
}

#[test]
fn write_file_wrong_arity_is_an_arity_error() {
    let error = eval_program(r#"(write-file "only-a-path.txt")"#, &mut Session::default())
        .expect_err("write-file with one argument must fail named, not panic");
    assert_eq!(error.kind, ErrorKind::Arity);
}
