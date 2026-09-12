use my_lisp::{eval_program, load_core_library, Session};
use std::collections::BTreeMap;

const DOCS_INDEX: &str = include_str!("../../../lib/surface/uk-docs.wsm");
const UK_SURFACE: &str = include_str!("../../../lib/surface/uk.my");

// vsi_stable_ukrainski_nazvy_maiut_numeric_zapys_u_dovidnyku,
// dokumentatsiinyi_kliuch_ie_tilky_numeric,
// znak_pytannia_tochno_vidpovidaie_predykatam, and
// znak_oklyku_tochno_vidpovidaie_mutatsii were pure registry/doc-sync
// text checks, relocated to `cargo xtask verify` per TEST-ARCHITECTURE-1
// step 4 — see crates/xtask/src/checks.rs.

fn documented() -> BTreeMap<(String, String), String> {
    DOCS_INDEX
        .lines()
        .filter_map(|line| {
            let fields = line.split_whitespace().collect::<Vec<_>>();
            if fields.first() != Some(&"(doc") {
                return None;
            }
            assert!(
                fields.len() >= 5,
                "рядок документації має містити category numeric-ID UK kind: {line}"
            );
            Some((
                (fields[2].to_string(), fields[3].to_string()),
                fields[4].to_string(),
            ))
        })
        .collect()
}

#[test]
fn stari_nazvy_dvokh_predykativ_lyshaiutsia_aliasamy_symisnosti() {
    for binding in [
        "(define конфлікт? check-conflict)",
        "(define перевірити-конфлікт check-conflict)",
        "(define змінна-зустрічається? occurs-check)",
        "(define перевірити-зустрічання occurs-check)",
    ] {
        assert!(UK_SURFACE.contains(binding), "відсутній compatibility alias: {binding}");
    }
}

#[test]
fn dovidnyk_poiasniuie_ne_predykaty_shcho_mozhut_povernuty_pustyi_spysok() {
    for name in ["отримати-з-карти", "підтримувальний-доказ", "та", "або"] {
        let kind = documented()
            .into_iter()
            .find_map(|((_id, uk), kind)| (uk == name).then_some(kind))
            .unwrap_or_else(|| panic!("немає документаційного запису для {name}"));
        assert_ne!(kind, "predicate");
        assert!(!name.ends_with('?'));
    }
}

fn uk_session() -> Session {
    let mut session = Session::default();
    load_core_library(&mut session).expect("core bootstrap");
    for source in [
        include_str!("../../../lib/unify.my"),
        include_str!("../../../lib/reason.my"),
        include_str!("../../../lib/forward.my"),
        include_str!("../../../lib/knowledge.my"),
        include_str!("../../../lib/persistent-map.my"),
        include_str!("../../../lib/persistent-vector.my"),
        include_str!("../../../lib/time.my"),
        include_str!("../../../lib/epistemic.my"),
    ] {
        eval_program(source, &mut session).expect("передумови UK surface мають завантажитися");
    }
    eval_program(UK_SURFACE, &mut session).expect("українська поверхня має завантажитися");
    session
}

#[test]
fn istina_i_khyba_ie_imenamy_tyh_samykh_kanonichnykh_znachen() {
    let mut session = uk_session();
    assert_eq!(eval_program("істина", &mut session).unwrap().value.to_string(), "t");
    assert_eq!(eval_program("хиба", &mut session).unwrap().value.to_string(), "()");
}

#[test]
fn novi_predykatni_nazvy_i_stari_aliasy_vykonuiutsia_odnakovo() {
    let mut session = uk_session();
    for source in [
        "(конфлікт? 'невідомий '())",
        "(перевірити-конфлікт 'невідомий '())",
    ] {
        assert_eq!(eval_program(source, &mut session).unwrap().value.to_string(), "()");
    }
    for source in [
        "(змінна-зустрічається? (логічна-змінна 'x) '(f (var x)) '())",
        "(перевірити-зустрічання (логічна-змінна 'x) '(f (var x)) '())",
    ] {
        assert_eq!(eval_program(source, &mut session).unwrap().value.to_string(), "t");
    }
}

// smyslovyi_audyt_pokryvaie_vsi_140_stable_nazv,
// seredovyshche_ne_maie_povtornoho_surface_binding, and
// stari_nazvy_smystovoho_audytu_lyshaiutsia_aliasamy_sumisnosti were pure
// registry/doc-sync text checks, relocated to `cargo xtask verify` per
// TEST-ARCHITECTURE-1 step 4 — see crates/xtask/src/checks.rs.
