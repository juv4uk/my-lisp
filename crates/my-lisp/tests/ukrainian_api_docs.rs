use my_lisp::{eval_program, load_core_library, Session};
use std::collections::{BTreeMap, BTreeSet};

const REGISTRY: &str = include_str!("../../../lib/surface/semantic-registry.wsm");
const DOCS_INDEX: &str = include_str!("../../../lib/surface/uk-docs.wsm");
const DOCS_MD: &str = include_str!("../../../docs/ukrainian-api.md");
const UK_SURFACE: &str = include_str!("../../../lib/surface/uk.my");
const NAME_AUDIT: &str = include_str!("../../../lib/surface/uk-name-audit.wsm");

fn stable_pairs() -> BTreeSet<(String, String)> {
    REGISTRY
        .lines()
        .filter_map(|line| {
            let fields = line.split_whitespace().collect::<Vec<_>>();
            let identity = fields.first()?.strip_prefix('(')?;
            if identity.len() < 4 || !identity.chars().all(|ch| ch.is_ascii_digit()) {
                return None;
            }
            let uk_index = fields.iter().position(|field| *field == "(uk")?;
            let uk = *fields.get(uk_index + 1)?;
            let status = fields.get(uk_index + 2)?.trim_end_matches(')');
            (status == "stable").then(|| (identity.to_string(), uk.to_string()))
        })
        .collect()
}

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
fn vsi_stable_ukrainski_nazvy_maiut_numeric_zapys_u_dovidnyku() {
    let coverage = stable_pairs();
    let docs = documented();

    assert_eq!(coverage.len(), 140, "stable UK-покриття змінилося");
    assert_eq!(docs.len(), 140, "довідник мусить мати рівно 140 записів");
    assert_eq!(
        coverage,
        docs.keys().cloned().collect::<BTreeSet<_>>(),
        "numeric registry і український документаційний індекс розійшлися"
    );

    for (_, uk) in &coverage {
        assert!(
            DOCS_MD.contains(&format!("| `{uk}` |")),
            "публічне українське ім'я відсутнє у Markdown-довіднику: {uk}"
        );
    }
}

#[test]
fn dokumentatsiinyi_kliuch_ie_tilky_numeric() {
    for ((identity, _uk), _) in documented() {
        assert!(
            identity.len() >= 4 && identity.chars().all(|ch| ch.is_ascii_digit()),
            "документаційний join key не може бути EN spelling: {identity}"
        );
    }
}

#[test]
fn znak_pytannia_tochno_vidpovidaie_predykatam() {
    let docs = documented();
    let mut predicates = 0usize;
    for ((_id, uk), kind) in docs {
        let question_name = uk.ends_with('?');
        let predicate = kind == "predicate";
        assert_eq!(
            question_name, predicate,
            "українська назва {uk}: знак ? і predicate мусять збігатися"
        );
        predicates += usize::from(predicate);
    }
    assert_eq!(predicates, 30, "змінився каталог публічних предикатів");
}

#[test]
fn znak_oklyku_tochno_vidpovidaie_mutatsii() {
    let docs = documented();
    let mut mutations = 0usize;
    for ((_id, uk), kind) in docs {
        let mutation = kind == "mutation";
        assert_eq!(uk.ends_with('!'), mutation, "{uk}: ! зарезервований для мутації");
        if mutation {
            mutations += 1;
            assert_eq!(uk, "встановити-елемент-вектора!");
        }
    }
    assert_eq!(mutations, 1, "публічний каталог мутацій змінився");
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

#[test]
fn smyslovyi_audyt_pokryvaie_vsi_140_stable_nazv() {
    assert!(NAME_AUDIT.contains("(stable-reviewed 140)"));
    assert!(NAME_AUDIT.contains("(renamed 42)"));
    assert!(NAME_AUDIT.contains("(retained 98)"));
    assert_eq!(
        NAME_AUDIT
            .lines()
            .filter(|line| line.trim_start().starts_with("(rename "))
            .count(),
        42
    );
}

#[test]
fn seredovyshche_ne_maie_povtornoho_surface_binding() {
    let needle = "(define середовище env)";
    assert_eq!(UK_SURFACE.match_indices(needle).count(), 1);
}

#[test]
fn stari_nazvy_smystovoho_audytu_lyshaiutsia_aliasamy_sumisnosti() {
    for line in NAME_AUDIT.lines() {
        let fields = line.split_whitespace().collect::<Vec<_>>();
        if fields.first() != Some(&"(rename") {
            continue;
        }
        let en = fields[1];
        let old = fields[2];
        let new = fields[3];
        assert!(UK_SURFACE.contains(&format!("(define {old} {en})")));
        assert!(UK_SURFACE.contains(&format!("(define {new} {en})")));
    }
}
