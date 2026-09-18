use my_lisp::{eval_program, Session};

const EXPERIMENT: &str = include_str!("../../../experiments/empty-list-unfold.lisp");

#[test]
fn empty_list_ground_unfolds_one_table_row_as_plain_lisp_data() {
    let mut session = Session::default();

    let source = format!("{EXPERIMENT}\nempty-list-unfold-result");
    let result = eval_program(&source, &mut session)
        .expect("the empty-list unfolding witness must execute as ordinary Lisp");

    assert_eq!(
        result.value.to_string(),
        "((00000000 (en ()) (uk ()) (ukr ()) (sa ()) (sym ())))"
    );
}

#[test]
fn unfolding_is_structurally_idempotent() {
    let mut session = Session::default();

    let source = format!(
        "{EXPERIMENT}\n(eq (quote ((00000000 (en ()) (uk ()) (ukr ()) (sa ()) (sym ())))) empty-list-unfold-result)"
    );
    let result = eval_program(&source, &mut session)
        .expect("shape-preserving traversal must execute");

    // Do not assume universal t/nil: preserve the post-revolution explicit
    // identity relation produced by EQ.
    assert_eq!(result.value.to_string(), "(identity-relation same)");
}

#[test]
fn empty_list_recovers_its_opaque_identity_from_lisp_data() {
    let mut session = Session::default();

    let source = format!("{EXPERIMENT}\nempty-list-identity");
    let result = eval_program(&source, &mut session)
        .expect("empty list must recover its identity from the Lisp witness");

    assert_eq!(result.value.to_string(), "00000000");
}

#[test]
fn eight_bit_zero_is_not_numeric_zero() {
    let mut session = Session::default();

    let bits = eval_program("(quote 00000000)", &mut session)
        .expect("8-bit pattern must survive quote as data");
    assert_eq!(bits.value.to_string(), "00000000");

    let relation = eval_program("(eq (quote 00000000) 0)", &mut session)
        .expect("8-bit pattern and number are both atoms and may be compared");
    assert_eq!(relation.value.to_string(), "(identity-relation distinct)");
}

#[test]
fn eight_bit_pattern_survives_read_write_round_trip() {
    let mut session = Session::default();
    let result = eval_program("(write-to-string (read \"00000000\"))", &mut session)
        .expect("8-bit pattern must survive reader and printer without decimal reinterpretation");

    assert_eq!(result.value.to_string(), "\"00000000\"");
}
