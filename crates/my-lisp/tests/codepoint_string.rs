use my_lisp::{eval_program, ErrorKind, Session};

#[test]
fn codepoint_to_string_materializes_unicode_scalars() {
    let mut session = Session::default();
    for (scalar, expected) in [
        (65, "A"),
        (162, "¢"),
        (8364, "€"),
        (128512, "😀"),
    ] {
        let result = eval_program(&format!("(codepoint->string {scalar})"), &mut session)
            .expect("valid Unicode scalar should materialize");
        assert_eq!(result.value.to_string(), format!("\"{expected}\""));
    }
}

#[test]
fn codepoint_to_string_rejects_non_scalars_and_inexact_values() {
    for source in [
        "(codepoint->string -1)",
        "(codepoint->string 55296)",
        "(codepoint->string 1114112)",
        // Decimal literals are exact in my-lisp. JSON numbers are a real
        // source of inexact runtime values, so use that boundary here.
        "(codepoint->string (json-parse \"65.0\"))",
        "(codepoint->string 1/2)",
    ] {
        let error = eval_program(source, &mut Session::default())
            .expect_err("invalid scalar input must fail named");
        assert_eq!(error.kind, ErrorKind::Type, "source: {source}");
    }
}
