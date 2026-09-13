use my_lisp::semantic_registry_export::{
    admitted_surfaces_for_semantic_id, semantic_id_for_admitted_surface,
};

const STRING_EMPTY_ID: &str = "1045";
const CURRENT_UK_STRING_EMPTY: &str = "текст-порожній?";
const FULL_UK_STRING_EMPTY: &str = "порожній-текст?";

#[test]
fn ratified_full_uk_name_resolves_to_same_identity_as_current_uk() {
    assert_eq!(
        semantic_id_for_admitted_surface(CURRENT_UK_STRING_EMPTY),
        Some(STRING_EMPTY_ID),
        "existing stable uk spelling must remain admitted"
    );
    assert_eq!(
        semantic_id_for_admitted_surface(FULL_UK_STRING_EMPTY),
        Some(STRING_EMPTY_ID),
        "ratified full-uk spelling must resolve to the same semantic identity"
    );

    let surfaces = admitted_surfaces_for_semantic_id(STRING_EMPTY_ID);
    assert!(
        surfaces.iter().any(|row| {
            row.namespace == "uk" && row.name == CURRENT_UK_STRING_EMPTY
        }),
        "current uk surface must remain present"
    );
    assert!(
        surfaces
            .iter()
            .any(|row| row.namespace == "full-uk" && row.name == FULL_UK_STRING_EMPTY),
        "full-uk must be represented as its own peer namespace"
    );
}

#[test]
fn admitted_full_uk_registry_spellings_never_require_latin_layout() {
    let source = include_str!("../../../lib/surface/semantic-registry.wsm");
    let mut admitted = 0usize;

    for line in source.lines() {
        let mut rest = line;
        while let Some(offset) = rest.find("(full-uk ") {
            rest = &rest[offset + "(full-uk ".len()..];
            let Some(end) = rest.find(')') else {
                panic!("unterminated full-uk surface entry: {line}");
            };
            let fields = rest[..end].split_whitespace().collect::<Vec<_>>();
            assert_eq!(
                fields.len(),
                2,
                "full-uk surface entry must have spelling and status: {line}"
            );
            let name = fields[0];
            let status = fields[1];
            if matches!(status, "stable" | "compatibility-only") && name != "—" {
                admitted += 1;
                assert!(
                    !name.chars().any(|character| character.is_ascii_alphabetic()),
                    "admitted full-uk spelling requires Latin layout: {name}"
                );
            }
            rest = &rest[end + 1..];
        }
    }

    assert!(
        admitted > 0,
        "registry must contain at least one admitted full-uk spelling"
    );
}

#[test]
fn generated_function_table_projects_authoritative_full_uk_name() {
    let table = include_str!("../../../lib/generated/function-table.wsm");
    let row = table
        .lines()
        .find(|line| line.trim_start().starts_with("(1045 "))
        .expect("generated function table must contain semantic ID 1045");

    assert!(
        row.contains("(uk текст-порожній? stable)"),
        "generated row must preserve current uk spelling: {row}"
    );
    assert!(
        row.contains("(full-uk порожній-текст? stable)"),
        "generated row must project the registry full-uk spelling instead of mirroring uk: {row}"
    );
}

#[test]
fn generated_function_table_exposes_ukr_as_an_additional_projection_column() {
    let table = include_str!("../../../lib/generated/function-table.wsm");
    let row = table
        .lines()
        .find(|line| line.trim_start().starts_with("(1045 "))
        .expect("generated function table must contain semantic ID 1045");

    let uk = row
        .find("(uk текст-порожній? stable)")
        .expect("row must contain current uk column");
    let full_uk = row
        .find("(full-uk порожній-текст? stable)")
        .expect("row must contain full-uk column");
    let ukr = row
        .find("(ukr порожній-текст? stable)")
        .expect("row must contain additional ukr projection column");
    let en = row
        .find("(en string-empty? stable)")
        .expect("row must contain English column");

    assert!(
        uk < full_uk && full_uk < ukr && ukr < en,
        "generated machine table column order must be uk -> full-uk -> ukr -> en: {row}"
    );

    let markdown = include_str!("../../../docs/generated/function-table.md");
    assert!(
        markdown.contains("| ID | Українська | Повна українська | ukr | full-uk status | English | Sanskrit | primary |"),
        "human function table must expose ukr as an additional column"
    );
    assert!(
        markdown.contains("| `1045` | текст-порожній? | порожній-текст? | порожній-текст? | stable | string-empty? |"),
        "human row 1045 must repeat the full Ukrainian name in the ukr projection"
    );
}
