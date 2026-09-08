use my_lisp::{
    eval_program, language_items, render_value_for_presentation, PresentationLanguage, Session, Value,
};
use std::rc::Rc;

const UK_SURFACE: &str = include_str!("../../../lib/surface/uk.my");
const SA_SURFACE: &str = include_str!("../../../lib/surface/sa.my");
const REGISTRY: &str = include_str!("../../../lib/surface/semantic-registry.wsm");
const PRESENTATION: &str = include_str!("../src/presentation.rs");

struct PeerCase {
    identity: &'static str,
    uk: &'static str,
    sa: &'static str,
    sym: &'static str,
    args: &'static str,
    expected: &'static str,
}

const CASES: &[PeerCase] = &[
    PeerCase {
        identity: "1001",
        uk: "відняти",
        sa: "viyoga",
        sym: "-",
        args: "44 2",
        expected: "42",
    },
    PeerCase {
        identity: "1002",
        uk: "помножити",
        sa: "guṇana",
        sym: "*",
        args: "6 7",
        expected: "42",
    },
    PeerCase {
        identity: "1003",
        uk: "поділити",
        sa: "haraṇa",
        sym: "/",
        args: "84 2",
        expected: "42",
    },
    PeerCase {
        identity: "1014",
        uk: "менше?",
        sa: "hīna?",
        sym: "<",
        args: "1 2",
        expected: "t",
    },
    PeerCase {
        identity: "1015",
        uk: "більше?",
        sa: "adhika?",
        sym: ">",
        args: "2 1",
        expected: "t",
    },
    PeerCase {
        identity: "1016",
        uk: "рівне?",
        sa: "sama?",
        sym: "=",
        args: "42 42",
        expected: "t",
    },
];

fn assert_same_builtin(left: &Value, right: &Value) {
    match (left, right) {
        (Value::Builtin(left), Value::Builtin(right)) => assert!(
            Rc::ptr_eq(left, right),
            "peer spellings must point to one builtin allocation"
        ),
        other => panic!("expected builtin peer values, got {other:?}"),
    }
}

fn value(session: &mut Session, source: &str) -> Value {
    eval_program(source, session)
        .unwrap_or_else(|error| panic!("{source}: {error:?}"))
        .value
}

#[test]
fn stable_operator_peers_exist_before_human_surface_libraries_load() {
    for case in CASES {
        let mut session = Session::default();
        let uk = value(&mut session, case.uk);
        let sa = value(&mut session, case.sa);
        let sym = value(&mut session, case.sym);

        assert_same_builtin(&uk, &sa);
        assert_same_builtin(&sa, &sym);

        for name in [case.uk, case.sa, case.sym] {
            assert_eq!(
                value(&mut session, &format!("({name} {})", case.args)).to_string(),
                case.expected,
                "{} / {name}",
                case.identity
            );
        }
    }
}

#[test]
fn shadowing_one_operator_spelling_never_retargets_its_peers() {
    for case in CASES {
        let names = [case.uk, case.sa, case.sym];
        for shadowed_index in 0..names.len() {
            let mut session = Session::default();
            let shadowed = names[shadowed_index];
            value(
                &mut session,
                &format!("(define {shadowed} (lambda (a b) (quote затінено)))"),
            );
            assert_eq!(
                value(&mut session, &format!("({shadowed} {})", case.args)).to_string(),
                "затінено"
            );

            for (index, peer) in names.iter().enumerate() {
                if index == shadowed_index {
                    continue;
                }
                assert_eq!(
                    value(&mut session, &format!("({peer} {})", case.args)).to_string(),
                    case.expected,
                    "{}: shadowing {shadowed} retargeted {peer}",
                    case.identity
                );
            }
        }
    }
}

#[test]
fn migrated_surface_files_do_not_build_stable_operator_peers_through_symbols() {
    for forbidden in [
        "(define відняти -)",
        "(define помножити *)",
        "(define поділити /)",
        "(define менше? <)",
        "(define більше? >)",
        "(define рівне? =)",
    ] {
        assert!(!UK_SURFACE.contains(forbidden), "UK alias leaked back: {forbidden}");
    }
    for forbidden in [
        "(define viyoga -)",
        "(define guṇana *)",
        "(define haraṇa /)",
        "(define hīna? <)",
        "(define adhika? >)",
        "(define sama? =)",
    ] {
        assert!(!SA_SURFACE.contains(forbidden), "SA alias leaked back: {forbidden}");
    }
}

#[test]
fn runtime_peer_slice_matches_numeric_registry_rows() {
    for case in CASES {
        assert!(REGISTRY.contains(&format!("({}", case.identity)));
        assert!(REGISTRY.contains(&format!("(uk {} stable)", case.uk)));
        assert!(REGISTRY.contains(&format!("(sa {} stable)", case.sa)));
        assert!(REGISTRY.contains(&format!("(sym {} stable)", case.sym)));
    }
}

#[test]
fn ukrainian_builtin_presentation_uses_numeric_authority_not_legacy_audit() {
    assert!(!PRESENTATION.contains("uk-sa-coverage.wsm"));
    assert!(PRESENTATION.contains("semantic-registry.wsm"));

    for case in CASES {
        let mut session = Session::default();
        let builtin = value(&mut session, case.sym);
        assert_eq!(
            render_value_for_presentation(&builtin, PresentationLanguage::Ukrainian),
            format!("#<вбудована {}>", case.uk),
            "{} presentation",
            case.identity
        );
    }
}

#[test]
fn tooling_metadata_follows_the_shared_builtin_value_for_every_peer() {
    let items = language_items();
    for case in CASES {
        let lookup = |name: &str| {
            items
                .iter()
                .find(|item| item.name == name)
                .unwrap_or_else(|| panic!("missing tooling item {name}"))
        };
        let uk = lookup(case.uk);
        let sa = lookup(case.sa);
        let sym = lookup(case.sym);
        assert_eq!(uk.signature, sym.signature, "{} UK signature", case.identity);
        assert_eq!(sa.signature, sym.signature, "{} SA signature", case.identity);
        assert_eq!(uk.documentation, sym.documentation, "{} UK docs", case.identity);
        assert_eq!(sa.documentation, sym.documentation, "{} SA docs", case.identity);
        assert_eq!(uk.arity, sym.arity, "{} UK arity", case.identity);
        assert_eq!(sa.arity, sym.arity, "{} SA arity", case.identity);
    }
}
