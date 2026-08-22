use my_lisp::{eval_program, Session};
use std::fs;

/// `read-file` is a host capability; installed here via the dev-dependency.
fn install_read_capability() {
    my_lisp_host::install();
}

#[test]
fn linter_gate() {
    install_read_capability();
    let mut session = Session::default();
    
    // Evaluate the libraries directly to load them into the environment
    let core_src = fs::read_to_string("../../lib/core.my").unwrap();
    eval_program(&core_src, &mut session).unwrap();
    let linter_src = fs::read_to_string("../../lib/linter.my").unwrap();
    eval_program(&linter_src, &mut session).unwrap();
    
    // Check if linter.my works by defining the threshold script
    let runner_src = r#"
        (def thresholds (quote ((max-size . 2000) (max-nesting . 30) (max-complexity . 24) (max-globals . 55) (max-effects . 20))))

        (def check-file-loop
          (lambda (path remaining all-violations)
            (cond
              ((equal? remaining (quote ())) all-violations)
              (t
               (let ((violations (lint-check (car remaining) thresholds)))
                 (cond
                   ((equal? violations (quote ())) (check-file-loop path (cdr remaining) all-violations))
                   (t (check-file-loop path (cdr remaining) (cons (list path (car remaining) violations) all-violations)))))))))

        (def check-file
          (lambda (path)
            (check-file-loop path (read-all (read-file path)) (quote ()))))

        (def append-all
          (lambda (lists)
            (cond
              ((equal? lists (quote ())) (quote ()))
              (t (append (car lists) (append-all (cdr lists)))))))

        (append-all
          (list
            (check-file "../../lib/core.my")
            (check-file "../../lib/linter.my")
            (check-file "../../lib/world.my")
            (check-file "../../lib/content-store.my")
            (check-file "../../lib/reason.my")))
    "#;
    
    
    match eval_program(runner_src, &mut session) {
        Ok(res) => {
            if !matches!(res.value, my_lisp::Value::Nil) {
                panic!("Lint violations found:\n{}", res.value);
            }
        }
        Err(e) => {
            panic!("Error evaluating runner_src:\n{:?}", e);
        }
    }
}
