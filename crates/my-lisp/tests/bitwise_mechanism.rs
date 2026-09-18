use my_lisp::{eval_program, ErrorKind, Session};

fn eval(source: &str) -> String {
    let mut session = Session::default();
    eval_program(source, &mut session)
        .unwrap_or_else(|error| panic!("{source}: {}", error.render(source)))
        .value
        .to_string()
}

fn error_kind(source: &str) -> ErrorKind {
    let mut session = Session::default();
    eval_program(source, &mut session)
        .expect_err("expression should fail")
        .kind
}

#[test]
fn raw_bitwise_mechanisms_match_small_integer_laws() {
    assert_eq!(eval("(integer-bit-and-raw 13 11)"), "9");
    assert_eq!(eval("(integer-bit-or-raw 8 3)"), "11");
    assert_eq!(eval("(integer-bit-xor-raw 15 6)"), "9");
    assert_eq!(eval("(integer-shift-left-raw 5 0)"), "5");
    assert_eq!(eval("(integer-shift-right-raw 5 0)"), "5");
}

#[test]
fn raw_bitwise_mechanisms_are_not_limited_to_u64() {
    assert_eq!(
        eval("(integer-shift-left-raw 1 80)"),
        "1208925819614629174706176"
    );
    assert_eq!(
        eval("(integer-bit-or-raw 1208925819614629174706176 3)"),
        "1208925819614629174706179"
    );
    assert_eq!(
        eval("(integer-bit-and-raw 1208925819614629174706179 7)"),
        "3"
    );
    assert_eq!(
        eval("(integer-bit-xor-raw 1208925819614629174706179 1208925819614629174706176)"),
        "3"
    );
    assert_eq!(
        eval("(integer-shift-right-raw 1208925819614629174706176 79)"),
        "2"
    );
}

#[test]
fn shifts_cover_machine_word_boundaries_without_making_them_language_limits() {
    assert_eq!(
        eval("(integer-shift-left-raw 1 63)"),
        "9223372036854775808"
    );
    assert_eq!(
        eval("(integer-shift-left-raw 1 64)"),
        "18446744073709551616"
    );
    assert_eq!(
        eval("(integer-shift-right-raw 18446744073709551616 64)"),
        "1"
    );
}

#[test]
fn raw_bitwise_mechanisms_reject_negative_and_noninteger_values() {
    for source in [
        "(integer-bit-and-raw -1 1)",
        "(integer-bit-or-raw 1 -1)",
        "(integer-bit-xor-raw 1 1/2)",
        "(integer-shift-left-raw -1 1)",
        "(integer-shift-left-raw 1 -1)",
        "(integer-shift-right-raw 1 1/2)",
    ] {
        assert_eq!(error_kind(source), ErrorKind::Type, "{source}");
    }
}

#[test]
fn raw_bitwise_mechanisms_keep_named_arity_errors() {
    assert_eq!(
        error_kind("(integer-bit-and-raw 1)"),
        ErrorKind::Arity
    );
    assert_eq!(
        error_kind("(integer-shift-left-raw 1)"),
        ErrorKind::Arity
    );
}

#[test]
fn raw_bitwise_results_respect_the_existing_numeric_bit_limit() {
    let mut session = Session::default();
    session.environment = session.environment.clone().with_numeric_bit_limit(64);
    let error = eval_program("(integer-shift-left-raw 1 64)", &mut session)
        .expect_err("65-bit result should exceed a 64-bit configured limit");
    assert_eq!(error.kind, ErrorKind::NumericOverflow);

    let ok = eval_program("(integer-shift-left-raw 1 63)", &mut session)
        .expect("64-bit result should fit the configured limit");
    assert_eq!(ok.value.to_string(), "9223372036854775808");
}
