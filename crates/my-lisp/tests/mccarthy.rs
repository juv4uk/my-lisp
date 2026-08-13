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

#[test]
fn bare_large_integer_literals_remain_exact() {
    let literal = "123456789012345678901234567890";
    assert_eq!(eval(literal).to_string(), literal);
    assert_eq!(eval(&format!("(+ {literal} 1)")).to_string(),
               "123456789012345678901234567891");
    assert_eq!(eval(&format!("(eq {literal} {literal})")), Value::Bool(true));
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
    assert!(duplicate.message.contains("povtornyi parametr"));

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
/// Variatyvni parametry (2026-08-09, prodovzhennia punktu 8 z PLAN.md): try
/// formy, spilni dlia rodyny Lisp, ne kliuchove slovo `&rest` odnoho
/// dialektu — `(a b . rest)` (dotted-spysok, ta sama pidtrymka readera,
/// dodana ranishe dlia literaliv danykh), holyi symvol (nul fiksovanykh
/// parametriv, kozhen arhument), i naiavnyi `(a b)` (tochna arnist, bez zmin).
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
/// `Display`/`print` ranishe pysaly `"{value}"` bez zhodnoho ekranuvannia —
/// riadok z bukvalnoiu `"` movchky lamav `read ∘ print = identity`
/// (nadrukovanyi tekst ne chytavsia nazad korektno: zakryvavsia zarano na
/// vbudovanii laptsi). Znaideno 2026-08-09 pid chas napysannia tulinhu, shcho
/// drukuie dani fikstur iz realnymy lapkamy. Vypravleno nadanniam `print`
/// spravzhnoi semantyky `prin1`/`write` (vlasna konventsiia Common
/// Lisp/Scheme dlia "bezpechnoi dlia read" funktsii druku): ekranuvaty `"`,
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
/// `princ` — «princ»/«display»-polovyna klasychnoi Lisp-pary funktsii druku,
/// druhu polovynu yakoi skladaie polahodzhenyi vyshche `print`: syryi tekst, bez
/// lapok i ekranuvannia, dlia vyvodu, pryznachenoho liudyni chy povtornomu
/// skladanniu yak bukvalnyi syrtsevyi tekst (napr. instrument, shcho heneruie
/// novyi `.my`-fail), nikoly ne dlia povtornoho parsynhu yak danykh.
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
/// `list` ranishe buv spetsialnoiu formoiu Rust; pereneseno v `lib/core.my`
/// toho samoho dnia, koly dodano variatyvni parametry lambda, bo `(def list
/// (lambda args args))` vyrazhaie tse tochno — toi samyi filtr G4/G5 ("chy
/// naiavne yadro vzhe mozhe tse skazaty?"), zastosovanyi do samoho Rust-sharu,
/// ne lyshe do `.my`-kodu.
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

#[test]
fn non_strict_comparisons_are_my_lisp_functions_not_rust_builtins() {
    let mut session = Session::default();
    eval_program(include_str!("../../../lib/core.my"), &mut session).unwrap();
    assert_eq!(eval_program("(<= 1 1 2)", &mut session).unwrap().value, Value::Bool(true));
    assert_eq!(eval_program("(<= 1 2 1)", &mut session).unwrap().value, Value::Nil);
    assert_eq!(eval_program("(>= 3 3 2)", &mut session).unwrap().value, Value::Bool(true));
    assert_eq!(eval_program("(>= 2 3)", &mut session).unwrap().value, Value::Nil);
    assert_eq!(eval_program("(<= 1/2 0.5)", &mut session).unwrap().value, Value::Bool(true));
    assert_eq!(eval_program("(<= 5)", &mut session).unwrap().value, Value::Bool(true));
    assert_eq!(eval_program("(<=)", &mut session).unwrap_err().kind, ErrorKind::Arity);
    assert_eq!(eval_program("(<= 1 2)", &mut Session::default()).unwrap_err().kind,
               ErrorKind::UnknownSymbol);
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
/// tests/fixtures/conformance.my — nezalezhnyi vid realizatsii kontrakt
/// (dyv. CLAUDE.md): bud-yaka maibutnia realizatsiia my-lisp — C, HDL, shcho
/// zavhodno — maie vidtvoriuvaty tsi rezultaty, shchoino pravylno realizuie sim
/// prymityviv i lambda/def/defmacro, bo vse, shcho nad nymy (vkliuchno z
/// lib/core.my), — zvychainyi my-lisp-kod, ne Rust. Poperednie zavantazhennia
/// core.my tut dozvoliaie fiksturam napriamu yoho vykorystovuvaty zamist
/// dubliuvannia pokryttia stdlib. Zapysano yak my-lisp-dani (2026-08-09,
/// pereneseno z JSON), tozh tsei test chytaie fail cherez `parse` — toi samyi
/// reader, kriz yakyi prokhodyt bud-yaka my-lisp-prohrama — ne cherez
/// `serde_json`; failu fikstur bilshe ne potriben chuzhyi format, shchob
/// lyshatys nezalezhnym vid realizatsii, yomu potriben vlasnyi reader
/// my-lisp, yakyi bud-yaka konformna realizatsiia vzhe maie za vyznachenniam.
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
    eval_program(include_str!("../../../lib/persistent-map.my"), &mut session)
        .expect("lib/persistent-map.my should load before conformance fixtures run");

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
fn symbol_predicate_is_a_my_lisp_function_not_a_rust_builtin() {
    let mut session = Session::default();
    eval_program(include_str!("../../../lib/core.my"), &mut session).unwrap();
    assert_eq!(
        eval_program("(symbol? 'hello)", &mut session)
            .unwrap()
            .value,
        Value::Bool(true)
    );
    assert_eq!(
        eval_program("(symbol? 5)", &mut session).unwrap().value,
        Value::Nil
    );
    assert_eq!(
        eval_program("(symbol? \"hello\")", &mut session)
            .unwrap()
            .value,
        Value::Nil
    );
    assert_eq!(
        eval_program("(symbol? '(hello))", &mut session)
            .unwrap()
            .value,
        Value::Nil
    );
    assert_eq!(
        eval_program(
            "(symbol? (string->symbol \"strange symbol\"))",
            &mut session
        )
        .unwrap()
        .value,
        Value::Bool(true)
    );
    assert_eq!(
        eval_program("(symbol? 'hello)", &mut Session::default())
            .unwrap_err()
            .kind,
        ErrorKind::UnknownSymbol
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
/// `my-lisp-constitution.my` — tse *zhenerovana proektsiia* nad
/// `tests/fixtures/conformance.my` (perehenerovuie `scripts/build-constitution.my`)
/// — ta sama forma "odne dzherelo + proektsiia", yaku `*knowledge-journal*`
/// z `lib/knowledge.my` vykorystovuie dlia rantaim-stanu, zastosovana tut do
/// dokumentatsii. Tsei test — prymusova CI-polovyna toho paternu: yakshcho khtos
/// dodast fiksturu v `conformance.my` i zabude pereheneruvaty, tsi dva
/// faily movchky roziidutsia — tsei test peretvoriuie tse na nehainyi, huchnyi
/// proval. Obydva faily teper my-lisp-dani (2026-08-09, pereneseno z JSON),
/// tozh tsei test parsyt yikh tak samo, yak `conformance_tests_from_my` vyshche,
/// ne cherez `serde_json`.
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
/// Pryntsyp proiektu 3 ("realizuvaty rozumnu mashynu") svidomo ne maie
/// vidpovidnyka sered G/S aksiom u `docs/language-core-axioms.md` — aksioma
/// tse tverdzhennia pro movu, tsei pryntsyp — tverdzhennia pro te, chomu proiekt
/// isnuie, i tse rizni katehorii navmysno. Ale tse oznachaie, shcho nishcho v samomu
/// movnomu kontrakti ne pomityt, yakshcho `lib/unify.my`/`lib/reason.my` tykho
/// vydaliat, abo pokryttia Rivnia 3 z chasom zmenshytsia — eroziiu vpiimaie lyshe
/// toi, khto vypadkovo zhadaie podyvytys. Tsei test — protsesna harantiia, ne
/// semantychna: vin ne pereviriaie, shcho oznachaiut `unify`/`reason` (tse robliat
/// `tests/unify.rs`/`tests/reason.rs`), lyshe shcho vony y dosi isnuiut,
/// zavantazhuiutsia, dovodiat odyn realnyi fakt, i shcho Riven 3 movchky ne
/// prosiv nyzhche mezhi. Yakshcho mezhu svidomo znyzhuiut — znyzyty tsiu perevirku
/// yavno, ne daty yii rozmytys nepomichenoiu.
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
/// S3 nazvav `OutOfMemory` u vlasnomu teksti do toho, yak katehoriia
/// isnuvala v kodi (znaideno pid chas audytu aksiom pered ratyfikatsiieiu,
/// 2026-08-09) — tsei test robyt yii realnoiu: optsiina mezha na kilkist
/// cons-komirok, shcho imituie spravdi obmezhenu kupu (vlasnyi pryklad S3,
/// "4096 cons-komirok na FPGA") bez potreby v realnomu zalizi, shchob
/// pereviryty tverdzhennia "obmezheni realizatsii provaliuiutsia nazvano,
/// nikoly ne pereoznachaiut sens `cons` movchky". Typova sesiia (kozhna
/// fikstura `conformance.my`) lyshaietsia neobmezhenoiu — tse optsiino, ne nova
/// typova mezha dlia etalonnoi realizatsii.
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
/// Ta sama forma, shcho y mezha `cons` vyshche, dlia vlasnoho nazvanoho prykladu
/// `S1` (`NumericOverflow`) zamist `S3` (`OutOfMemory`) — optsiina mezha v
/// bitakh na rezultaty tochnoi aryfmetyky. Nikoly ne vidkochuietsia do
/// netochnoho nablyzhennia za mezheiu (tse porushylo b S1, ne zadovolnylo b
/// yoho) — natomist provaliuietsia nazvano.
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
fn write_to_string_round_trips_structured_data_without_printing() {
    let mut session = Session::default();
    let result = eval_program(r#"(write-to-string '(package "radio" 3/2))"#, &mut session)
        .expect("structured data should serialize");
    assert_eq!(result.value, Value::String(r#"(package "radio" 3/2)"#.into()));
    assert!(result.output.is_empty());
    let reread = eval_program(r#"(read (write-to-string '(package "radio" 3/2)))"#, &mut session)
        .expect("serialized data should read back");
    assert_eq!(reread.value.to_string(), r#"(package "radio" 3/2)"#);
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

// --- write-file-bytes / read-file-bytes (PLAN.md item 22) -----------------
// The byte-level counterpart to write-file/read-file: write-file can only
// ever produce valid UTF-8 (Value::String wraps &str), so a byte like 0xff
// or 0xfe — not valid on its own as UTF-8 — could never be written raw.
// These round-trip a list of fixnums 0-255 through std::fs::write/read
// directly over Vec<u8>, proving non-UTF-8 bytes survive intact.

#[test]
fn write_file_bytes_then_read_file_bytes_round_trips_non_utf8_bytes() {
    let path = std::env::temp_dir().join("my-lisp-write-file-bytes-round-trip.bin");
    let path_str = path.to_str().expect("temp path should be valid UTF-8").replace('\\', "/");
    // 255 and 254 are not valid standalone UTF-8 bytes — this is the exact
    // case write-file (String-based) cannot represent.
    let source = format!(r#"(write-file-bytes "{path_str}" '(0 1 2 255 65 254))"#);
    let mut session = Session::default();
    let result = eval_program(&source, &mut session).expect("write-file-bytes should succeed");
    assert_eq!(
        result.value,
        Value::list([0, 1, 2, 255, 65, 254].map(|n| Value::Number(n as f64, Exactness::Exact)))
    );

    let raw = std::fs::read(&path).expect("the file should exist with raw bytes");
    assert_eq!(raw, vec![0u8, 1, 2, 255, 65, 254]);

    let read_back = eval_program(&format!(r#"(read-file-bytes "{path_str}")"#), &mut session)
        .expect("read-file-bytes should read back what write-file-bytes wrote");
    assert_eq!(
        read_back.value,
        Value::list([0, 1, 2, 255, 65, 254].map(|n| Value::Number(n as f64, Exactness::Exact)))
    );

    std::fs::remove_file(&path).ok();
}

#[test]
fn write_file_bytes_rejects_a_non_string_path() {
    let error = eval_program(r#"(write-file-bytes 42 '(1 2 3))"#, &mut Session::default())
        .expect_err("a non-string path must fail named, not panic");
    assert_eq!(error.kind, ErrorKind::Type);
}

#[test]
fn write_file_bytes_rejects_a_non_list_second_argument() {
    let error = eval_program(r#"(write-file-bytes "path-does-not-matter.bin" 42)"#, &mut Session::default())
        .expect_err("a non-list second argument must fail named, not panic");
    assert_eq!(error.kind, ErrorKind::Type);
}

#[test]
fn write_file_bytes_rejects_an_out_of_range_element() {
    let error = eval_program(r#"(write-file-bytes "path-does-not-matter.bin" '(1 256 3))"#, &mut Session::default())
        .expect_err("an element above 255 must fail named, not panic");
    assert_eq!(error.kind, ErrorKind::Type);
}

#[test]
fn write_file_bytes_rejects_a_negative_element() {
    let error = eval_program(r#"(write-file-bytes "path-does-not-matter.bin" '(1 -1 3))"#, &mut Session::default())
        .expect_err("a negative element must fail named, not panic");
    assert_eq!(error.kind, ErrorKind::Type);
}

#[test]
fn read_file_bytes_rejects_a_non_string_path() {
    let error = eval_program(r#"(read-file-bytes 42)"#, &mut Session::default())
        .expect_err("a non-string path must fail named, not panic");
    assert_eq!(error.kind, ErrorKind::Type);
}

#[test]
fn write_file_bytes_wrong_arity_is_an_arity_error() {
    let error = eval_program(r#"(write-file-bytes "only-a-path.bin")"#, &mut Session::default())
        .expect_err("write-file-bytes with one argument must fail named, not panic");
    assert_eq!(error.kind, ErrorKind::Arity);
}

#[test]
fn read_file_bytes_wrong_arity_is_an_arity_error() {
    let error = eval_program(r#"(read-file-bytes)"#, &mut Session::default())
        .expect_err("read-file-bytes with no arguments must fail named, not panic");
    assert_eq!(error.kind, ErrorKind::Arity);
}

// --- string-append (PLAN.md item 14) -------------------------------------

#[test]
fn string_append_concatenates_two_strings() {
    assert_eq!(
        eval(r#"(string-append "hello, " "world")"#),
        Value::String("hello, world".into())
    );
}

#[test]
fn string_append_rejects_a_non_string_first_argument() {
    let error = eval_program(r#"(string-append 1 "x")"#, &mut Session::default())
        .expect_err("a non-string first argument must fail named, not panic");
    assert_eq!(error.kind, ErrorKind::Type);
}

#[test]
fn string_append_rejects_a_non_string_second_argument() {
    let error = eval_program(r#"(string-append "x" 1)"#, &mut Session::default())
        .expect_err("a non-string second argument must fail named, not panic");
    assert_eq!(error.kind, ErrorKind::Type);
}

#[test]
fn string_append_wrong_arity_is_an_arity_error() {
    let error = eval_program(r#"(string-append "only-one")"#, &mut Session::default())
        .expect_err("string-append with one argument must fail named, not panic");
    assert_eq!(error.kind, ErrorKind::Arity);
}

// --- string<? (PLAN.md item 15 — the one primitive its persistent-map
// design needed) --------------------------------------------------------

#[test]
fn string_less_than_orders_strings_lexicographically() {
    assert_eq!(eval(r#"(string<? "a" "b")"#), Value::Bool(true));
    assert_eq!(eval(r#"(string<? "b" "a")"#), Value::Bool(false));
    assert_eq!(eval(r#"(string<? "a" "a")"#), Value::Bool(false));
}

#[test]
fn string_less_than_rejects_non_string_arguments() {
    let left = eval_program(r#"(string<? 1 "a")"#, &mut Session::default())
        .expect_err("a non-string left argument must fail named, not panic");
    assert_eq!(left.kind, ErrorKind::Type);

    let right = eval_program(r#"(string<? "a" 1)"#, &mut Session::default())
        .expect_err("a non-string right argument must fail named, not panic");
    assert_eq!(right.kind, ErrorKind::Type);
}

#[test]
fn string_less_than_wrong_arity_is_an_arity_error() {
    let error = eval_program(r#"(string<? "only-one")"#, &mut Session::default())
        .expect_err("string<? with one argument must fail named, not panic");
    assert_eq!(error.kind, ErrorKind::Arity);
}

// --- process-run (PLAN.md item 21's follow-up) ---------------------------
// Deliberately narrow: no shell (Command::new(program).args(args), never
// sh -c), and disabled by default — a session must opt in via
// Environment::with_process_allowlist, never something a my-lisp program
// can grant itself. See that method's own comment: combined with
// tcp-accept's inbound networking, an unrestricted process-run would let
// a remote peer reach arbitrary command execution.

#[test]
fn process_run_fails_named_when_the_session_never_opted_in() {
    let error = eval_program(r#"(process-run "git" (list "--version"))"#, &mut Session::default())
        .expect_err("process-run on the default (unrestricted-by-default-off) session must fail named, not run anything");
    assert_eq!(error.kind, ErrorKind::InvalidForm);
}

#[test]
fn process_run_succeeds_for_an_explicitly_allowed_program() {
    let mut session = Session {
        environment: Environment::root().with_process_allowlist(vec!["git".to_string()]),
    };
    // `git --version` runs on every platform this project builds for
    // (Linux/macOS/Windows CI runners all have `git` — the workflow
    // itself checks out the repo with it) without going through a
    // platform-specific shell (`cmd` on Windows, `sh`/`echo` elsewhere
    // aren't the same program), while still proving args are passed
    // through without a shell interpreting them as one string.
    let source = r#"(process-run "git" (quote ("--version")))"#;
    let result = eval_program(source, &mut session).expect("an explicitly allowed program should run");
    let Value::Pair(ref exit_code, ref rest) = result.value else {
        panic!("process-run should return a 3-element list");
    };
    assert_eq!(**exit_code, Value::Number(0.0, Exactness::Exact));
    let Value::Pair(ref stdout, _) = **rest else {
        panic!("process-run should return a 3-element list");
    };
    let Value::String(ref stdout) = **stdout else {
        panic!("stdout should be a string");
    };
    assert!(stdout.contains("git version"), "expected stdout to contain 'git version', got {stdout:?}");
}

#[test]
fn process_run_rejects_a_program_not_on_the_allowlist() {
    let mut session = Session {
        environment: Environment::root().with_process_allowlist(vec!["git".to_string()]),
    };
    let error = eval_program(r#"(process-run "cmd" (quote ("/C" "echo" "hi")))"#, &mut session)
        .expect_err("a program not on the allowlist must fail named, not run");
    assert_eq!(error.kind, ErrorKind::InvalidForm);
}

#[test]
fn process_run_rejects_a_non_string_program() {
    let mut session = Session {
        environment: Environment::root().with_process_allowlist(vec!["git".to_string()]),
    };
    let error = eval_program("(process-run 42 (list \"x\"))", &mut session)
        .expect_err("a non-string program name must fail named, not panic");
    assert_eq!(error.kind, ErrorKind::Type);
}

#[test]
fn process_run_rejects_a_non_list_args_argument() {
    let mut session = Session {
        environment: Environment::root().with_process_allowlist(vec!["git".to_string()]),
    };
    let error = eval_program(r#"(process-run "git" "not-a-list")"#, &mut session)
        .expect_err("a non-list args argument must fail named, not panic");
    assert_eq!(error.kind, ErrorKind::Type);
}

#[test]
fn process_run_rejects_a_non_string_element_in_args() {
    let mut session = Session {
        environment: Environment::root().with_process_allowlist(vec!["git".to_string()]),
    };
    let error = eval_program("(process-run \"git\" (quote (42)))", &mut session)
        .expect_err("a non-string element in args must fail named, not panic");
    assert_eq!(error.kind, ErrorKind::Type);
}

#[test]
fn process_run_wrong_arity_is_an_arity_error() {
    let mut session = Session {
        environment: Environment::root().with_process_allowlist(vec!["git".to_string()]),
    };
    let error = eval_program(r#"(process-run "git")"#, &mut session)
        .expect_err("process-run with one argument must fail named, not panic");
    assert_eq!(error.kind, ErrorKind::Arity);
}
