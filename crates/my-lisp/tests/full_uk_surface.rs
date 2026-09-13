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
