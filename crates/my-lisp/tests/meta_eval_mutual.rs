use my_lisp::{eval_program, Session};

fn eval_meta_program(program_source: &str, probe_source: &str) -> String {
    let mut session = Session::default();
    eval_program(include_str!("../../../lib/core.my"), &mut session).unwrap();
    eval_program(include_str!("../../../lib/meta-eval.my"), &mut session).unwrap();

    let source = format!(
        r#"(let ((loaded (my-eval-program (read-all "{}") (quote ()))))
             (my-eval (read "{}") (car loaded)))"#,
        program_source.replace('\\', "\\\\").replace('"', "\\\""),
        probe_source.replace('\\', "\\\\").replace('"', "\\\""),
    );

    eval_program(&source, &mut session)
        .unwrap()
        .value
        .to_string()
}

fn eval_native(program_source: &str, probe_source: &str) -> String {
    let mut session = Session::default();
    let source = format!("{program_source} {probe_source}");
    eval_program(&source, &mut session)
        .unwrap()
        .value
        .to_string()
}

#[test]
fn consecutive_top_level_functions_can_refer_to_each_other_in_main_meta_eval() {
    let program = r#"
(def even?
  (lambda (n)
    (cond
      ((eq n 0) t)
      (t (odd? (- n 1))))))
(def odd?
  (lambda (n)
    (cond
      ((eq n 0) (quote ()))
      (t (even? (- n 1))))))
"#;

    for probe in ["(even? 20)", "(odd? 21)", "(even? 19)", "(odd? 20)"] {
        let via_meta = eval_meta_program(program, probe);
        let via_native = eval_native(program, probe);
        assert_eq!(via_meta, via_native, "mutual-recursion parity failed for {probe}");
    }
}

#[test]
fn three_member_recursive_group_is_finite_lisp_data_in_main_meta_eval() {
    let program = r#"
(def mod0?
  (lambda (n)
    (cond
      ((eq n 0) t)
      (t (mod1? (- n 1))))))
(def mod1?
  (lambda (n)
    (cond
      ((eq n 0) (quote ()))
      (t (mod2? (- n 1))))))
(def mod2?
  (lambda (n)
    (cond
      ((eq n 0) (quote ()))
      (t (mod0? (- n 1))))))
"#;

    for probe in ["(mod0? 30)", "(mod1? 31)", "(mod2? 32)"] {
        let via_meta = eval_meta_program(program, probe);
        let via_native = eval_native(program, probe);
        assert_eq!(via_meta, via_native, "three-member group parity failed for {probe}");
    }
}
