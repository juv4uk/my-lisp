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
