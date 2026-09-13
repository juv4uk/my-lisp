const README: &str = include_str!("../../../README.md");
const UKRAINIAN_API: &str = include_str!("../../../docs/ukrainian-api.md");
const DOCS_INDEX: &str = include_str!("../../../lib/surface/uk-docs.wsm");

#[test]
fn readme_explains_the_two_ukrainian_surfaces() {
    for required in [
        "`uk` — коротка, інтуїтивно зрозуміла українська поверхня",
        "`ukr` — повна українська поверхня",
        "docs/generated/function-table.md",
        "candidate",
    ] {
        assert!(README.contains(required), "README missing Ukrainian surface contract: {required}");
    }
}

#[test]
fn ukrainian_api_distinguishes_surface_names_from_function_meaning() {
    for required in [
        "`uk` — коротке ім'я",
        "`ukr` — повне ім'я",
        "semantic ID",
        "stable",
        "candidate",
        "generated/function-table.md",
    ] {
        assert!(
            UKRAINIAN_API.contains(required),
            "docs/ukrainian-api.md missing uk/ukr documentation rule: {required}"
        );
    }
}

#[test]
fn machine_docs_index_states_that_registry_owns_surface_spellings() {
    for required in [
        "numeric semantic ID",
        "uk/ukr",
        "semantic-registry.wsm",
        "опис функції не дублюється",
    ] {
        assert!(
            DOCS_INDEX.contains(required),
            "lib/surface/uk-docs.wsm missing authority note: {required}"
        );
    }
}

fn documented_ids() -> Vec<String> {
    DOCS_INDEX
        .lines()
        .filter_map(|line| {
            let line = line.trim_start();
            let rest = line.strip_prefix("(doc ")?;
            let mut fields = rest.split_whitespace();
            fields.next()?; // category
            let id = fields.next()?;
            id.chars().all(|character| character.is_ascii_digit()).then(|| id.to_string())
        })
        .collect()
}

#[test]
fn every_documented_identity_shows_uk_ukr_status_and_one_behavior_description() {
    let (_, reference_and_tail) = UKRAINIAN_API
        .split_once("## Повний довідник")
        .expect("Ukrainian API must contain the detailed reference section");
    let (reference, _) = reference_and_tail
        .split_once("## Межа довідника")
        .expect("Ukrainian API detailed reference must end before the boundary section");

    assert!(
        reference.contains("| semantic ID | `uk` | `ukr` | статус `ukr` | Виклик | Тип | Що робить | Основа |"),
        "detailed Ukrainian API must expose uk and ukr side by side"
    );

    let ids = documented_ids();
    assert_eq!(ids.len(), 140, "machine docs index must still document 140 stable uk identities");

    let rows = reference
        .lines()
        .filter(|line| line.starts_with("| `") && line.chars().filter(|character| *character == '|').count() == 9)
        .collect::<Vec<_>>();
    assert_eq!(
        rows.len(),
        ids.len(),
        "detailed reference must contain exactly one eight-column row per documented semantic ID"
    );

    for id in ids {
        let marker = format!("| `{id}` |");
        let count = rows.iter().filter(|row| row.starts_with(&marker)).count();
        assert_eq!(
            count, 1,
            "semantic ID {id} must appear exactly once in the detailed uk/ukr reference"
        );
    }

    for required_row_fragment in [
        "| `1045` | `текст-порожній?` | `порожній-текст?` | stable |",
        "| `1075` | `монотонний-нс` | `монотонний-час-у-наносекундах` | candidate |",
    ] {
        assert!(
            reference.contains(required_row_fragment),
            "detailed reference missing representative uk/ukr row: {required_row_fragment}"
        );
    }
}
