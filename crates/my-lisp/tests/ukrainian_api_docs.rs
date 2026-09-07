use std::collections::{BTreeMap, BTreeSet};

const COVERAGE: &str = include_str!("../../../lib/surface/uk-sa-coverage.wsm");
const DOCS_INDEX: &str = include_str!("../../../lib/surface/uk-docs.wsm");
const DOCS_MD: &str = include_str!("../../../docs/ukrainian-api.md");
const UK_SURFACE: &str = include_str!("../../../lib/surface/uk.my");

fn stable_pairs() -> BTreeSet<(String, String)> {
    COVERAGE
        .lines()
        .filter_map(|line| {
            let fields = line.split_whitespace().collect::<Vec<_>>();
            if fields.first() != Some(&"(entry") || fields.get(5) != Some(&"stable") {
                return None;
            }
            Some((fields[2].to_string(), fields[3].to_string()))
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
                "рядок документації має містити category EN UK kind: {line}"
            );
            Some(((fields[2].to_string(), fields[3].to_string()), fields[4].to_string()))
        })
        .collect()
}

#[test]
fn vsi_stable_ukrainski_nazvy_maiut_zapys_u_dovidnyku() {
    let coverage = stable_pairs();
    let docs = documented();

    assert_eq!(coverage.len(), 140, "stable-покриття змінилося");
    assert_eq!(docs.len(), 140, "довідник мусить мати рівно 140 записів");

    let doc_pairs = docs.keys().cloned().collect::<BTreeSet<_>>();
    assert_eq!(
        coverage, doc_pairs,
        "stable surface і машинний документаційний індекс розійшлися"
    );

    for (_, uk) in &coverage {
        assert!(
            DOCS_MD.contains(&format!("| `{uk}` |")),
            "публічне українське ім'я відсутнє у Markdown-довіднику: {uk}"
        );
    }
}

#[test]
fn znak_pytannia_tochno_vidpovidaie_predykatam() {
    let docs = documented();
    let mut predicates = 0usize;

    for ((_en, uk), kind) in docs {
        let question_name = uk.ends_with('?');
        let predicate = kind == "predicate";

        assert_eq!(
            question_name, predicate,
            "українська назва {uk}: знак ? і класифікація predicate мусять збігатися"
        );
        predicates += usize::from(predicate);
    }

    assert_eq!(predicates, 30, "змінився каталог публічних предикатів");
}

#[test]
fn znak_oklyku_tochno_vidpovidaie_mutatsii() {
    let docs = documented();
    let mut mutations = 0usize;

    for ((_en, uk), kind) in docs {
        let mutation = kind == "mutation";
        assert_eq!(
            uk.ends_with('!'),
            mutation,
            "українська назва {uk}: знак ! зарезервований для мутації"
        );
        if mutation {
            mutations += 1;
            assert_eq!(uk, "встановити-вектор!");
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
        assert!(
            UK_SURFACE.contains(binding),
            "відсутній очікуваний stable/compatibility alias: {binding}"
        );
    }
}

#[test]
fn dovidnyk_poiasniuie_ne_predykaty_shcho_mozhut_povernuty_pustyi_spysok() {
    for name in ["карта-отримати", "підтримуючий-доказ", "та", "або"] {
        let kind = documented()
            .into_iter()
            .find_map(|((_en, uk), kind)| (uk == name).then_some(kind))
            .unwrap_or_else(|| panic!("немає документаційного запису для {name}"));
        assert_ne!(kind, "predicate", "{name} повертає дані/значення, а не лише t/()");
        assert!(!name.ends_with('?'));
    }
}
