use my_lisp::{eval_program, Session};

#[test]
fn host_authored_semantic_expectation_is_forbidden() {
    let actual = eval_program("(car (quote (a b)))", &mut Session::default())
        .unwrap()
        .value
        .to_string();
    assert_eq!(actual, "a");
}
