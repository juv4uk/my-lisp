use my_lisp::{eval_program, Session, Value};
use std::rc::Rc;

const UK_SURFACE: &str = include_str!("../../../lib/surface/uk.my");
const SA_SURFACE: &str = include_str!("../../../lib/surface/sa.my");
const REGISTRY: &str = include_str!("../../../lib/surface/semantic-registry.wsm");

fn assert_same_builtin(left: &Value, right: &Value) {
    match (left, right) {
        (Value::Builtin(left), Value::Builtin(right)) => assert!(Rc::ptr_eq(left, right)),
        other => panic!("expected two builtin values, got {other:?}"),
    }
}

#[test]
fn add_peer_spellings_exist_before_any_human_surface_library_loads() {
    let mut session = Session::default();

    let uk = eval_program("додати", &mut session).expect("UK ADD").value;
    let en = eval_program("+", &mut session).expect("EN ADD").value;
    let sa = eval_program("yoga", &mut session).expect("SA ADD").value;

    assert_same_builtin(&uk, &en);
    assert_same_builtin(&en, &sa);
    assert_eq!(
        eval_program("(додати 20 22)", &mut session)
            .unwrap()
            .value
            .to_string(),
        "42"
    );
    assert_eq!(
        eval_program("(+ 20 22)", &mut session)
            .unwrap()
            .value
            .to_string(),
        "42"
    );
    assert_eq!(
        eval_program("(yoga 20 22)", &mut session)
            .unwrap()
            .value
            .to_string(),
        "42"
    );
}

#[test]
fn shadowing_one_add_spelling_does_not_retarget_its_peers() {
    for (shadowed, first_peer, second_peer) in [
        ("додати", "+", "yoga"),
        ("+", "додати", "yoga"),
        ("yoga", "додати", "+"),
    ] {
        let mut session = Session::default();
        eval_program(
            &format!("(define {shadowed} (lambda (a b) (quote shadowed)))"),
            &mut session,
        )
        .expect("ordinary peer spelling remains lexically shadowable");

        assert_eq!(
            eval_program(&format!("({shadowed} 1 2)"), &mut session)
                .unwrap()
                .value
                .to_string(),
            "shadowed"
        );
        assert_eq!(
            eval_program(&format!("({first_peer} 1 2)"), &mut session)
                .unwrap()
                .value
                .to_string(),
            "3"
        );
        assert_eq!(
            eval_program(&format!("({second_peer} 1 2)"), &mut session)
                .unwrap()
                .value
                .to_string(),
            "3"
        );
    }
}

#[test]
fn migrated_surface_files_no_longer_define_add_through_another_language() {
    assert!(!UK_SURFACE.contains("(define додати +)"));
    assert!(!SA_SURFACE.contains("(define yoga +)"));
    assert!(REGISTRY.contains("(m0104"));
    assert!(REGISTRY.contains("(uk додати stable)"));
    assert!(REGISTRY.contains("(en + stable)"));
    assert!(REGISTRY.contains("(sa yoga stable)"));
}
