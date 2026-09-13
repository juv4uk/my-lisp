use my_lisp::semantic_registry_export::{
    admitted_surfaces_for_semantic_id, semantic_id_for_admitted_surface,
};

const STRING_EMPTY_ID: &str = "1045";
const CURRENT_UK_STRING_EMPTY: &str = "текст-порожній?";
const UKR_STRING_EMPTY: &str = "порожній-текст?";

const CONCISE_UK_CASES: &[(&str, &str, &str)] = &[
    ("1031", "за-номером", "елемент-списку-за-індексом"),
    ("1032", "у-списку?", "значення-у-списку?"),
    ("1033", "за-ключем", "знайти-за-ключем"),
    ("1048", "текст-перший", "перший-символ-тексту"),
    ("1049", "текст-решта", "решта-символів-тексту"),
    ("1065", "новий-вектор", "створити-вектор"),
    ("1068", "вектор-встановити!", "встановити-елемент-вектора!"),
    ("1101", "вектор-за-номером", "елемент-вектора-за-індексом"),
];

const LEGACY_UK_CASES: &[(&str, &str)] = &[
    ("1031", "елемент-списку-за-індексом"),
    ("1032", "значення-у-списку?"),
    ("1033", "знайти-за-ключем"),
    ("1048", "перший-символ-тексту"),
    ("1049", "решта-символів-тексту"),
    ("1065", "створити-вектор"),
    ("1068", "встановити-елемент-вектора!"),
    ("1101", "елемент-вектора-за-індексом"),
];

#[test]
fn uk_is_concise_and_intuitive_while_ukr_keeps_the_full_wording() {
    for &(semantic_id, uk, ukr) in CONCISE_UK_CASES {
        assert_eq!(
            semantic_id_for_admitted_surface(uk),
            Some(semantic_id),
            "concise uk spelling must resolve through the same semantic identity: {uk}"
        );
        assert_eq!(
            semantic_id_for_admitted_surface(ukr),
            Some(semantic_id),
            "full ukr spelling must resolve through the same semantic identity: {ukr}"
        );

        let surfaces = admitted_surfaces_for_semantic_id(semantic_id);
        assert!(
            surfaces
                .iter()
                .any(|row| row.namespace == "uk" && row.name == uk),
            "semantic ID {semantic_id} must expose concise uk spelling {uk}"
        );
        assert!(
            surfaces
                .iter()
                .any(|row| row.namespace == "ukr" && row.name == ukr),
            "semantic ID {semantic_id} must expose full ukr spelling {ukr}"
        );
        assert!(
            uk.chars().count() < ukr.chars().count(),
            "reviewed concise uk spelling must actually be shorter than ukr: {uk} vs {ukr}"
        );
        assert!(
            !uk.chars().any(|character| character.is_ascii_alphabetic()),
            "concise uk spelling must stay on the Ukrainian keyboard layout: {uk}"
        );
    }
}

#[test]
fn replacing_a_verbose_uk_spelling_keeps_the_old_name_as_compatibility_surface() {
    for &(semantic_id, old_name) in LEGACY_UK_CASES {
        assert_eq!(
            semantic_id_for_admitted_surface(old_name),
            Some(semantic_id),
            "old stable uk spelling must remain admitted as compatibility alias: {old_name}"
        );
    }
}

#[test]
fn ratified_ukr_name_resolves_to_same_identity_as_current_uk() {
    assert_eq!(
        semantic_id_for_admitted_surface(CURRENT_UK_STRING_EMPTY),
        Some(STRING_EMPTY_ID),
        "existing stable uk spelling must remain admitted"
    );
    assert_eq!(
        semantic_id_for_admitted_surface(UKR_STRING_EMPTY),
        Some(STRING_EMPTY_ID),
        "ratified ukr spelling must resolve to the same semantic identity"
    );

    let surfaces = admitted_surfaces_for_semantic_id(STRING_EMPTY_ID);
    assert!(
        surfaces
            .iter()
            .any(|row| row.namespace == "uk" && row.name == CURRENT_UK_STRING_EMPTY),
        "current uk surface must remain present"
    );
    assert!(
        surfaces
            .iter()
            .any(|row| row.namespace == "ukr" && row.name == UKR_STRING_EMPTY),
        "ukr must be the authoritative full Ukrainian peer namespace"
    );
    assert!(
        surfaces.iter().all(|row| row.namespace != "full-uk"),
        "full-uk is not a separate namespace; ukr is the full Ukrainian surface"
    );
}

#[test]
fn admitted_ukr_registry_spellings_never_require_latin_layout() {
    let source = include_str!("../../../lib/surface/semantic-registry.wsm");
    let mut admitted = 0usize;

    assert!(
        !source.contains("(full-uk "),
        "registry must use ukr, not a duplicate full-uk namespace"
    );

    for line in source.lines() {
        let mut rest = line;
        while let Some(offset) = rest.find("(ukr ") {
            rest = &rest[offset + "(ukr ".len()..];
            let Some(end) = rest.find(')') else {
                panic!("unterminated ukr surface entry: {line}");
            };
            let fields = rest[..end].split_whitespace().collect::<Vec<_>>();
            assert_eq!(
                fields.len(),
                2,
                "ukr surface entry must have spelling and status: {line}"
            );
            let name = fields[0];
            let status = fields[1];
            if matches!(status, "stable" | "compatibility-only") && name != "—" {
                admitted += 1;
                assert!(
                    !name.chars().any(|character| character.is_ascii_alphabetic()),
                    "admitted ukr spelling requires Latin layout: {name}"
                );
            }
            rest = &rest[end + 1..];
        }
    }

    assert!(
        admitted > 0,
        "registry must contain at least one admitted ukr spelling"
    );
}

#[test]
fn generated_function_table_uses_uk_then_ukr_without_duplicate_full_uk_column() {
    let table = include_str!("../../../lib/generated/function-table.wsm");
    let row = table
        .lines()
        .find(|line| line.trim_start().starts_with("(1045 "))
        .expect("generated function table must contain semantic ID 1045");

    let uk = row
        .find("(uk текст-порожній? stable)")
        .expect("row must contain current uk column");
    let ukr = row
        .find("(ukr порожній-текст? stable)")
        .expect("row must contain authoritative full Ukrainian ukr column");
    let en = row
        .find("(en string-empty? stable)")
        .expect("row must contain English column");

    assert!(
        uk < ukr && ukr < en,
        "generated machine table column order must be uk -> ukr -> en: {row}"
    );
    assert!(
        !row.contains("(full-uk "),
        "generated machine table must not duplicate ukr as full-uk: {row}"
    );

    let markdown = include_str!("../../../docs/generated/function-table.md");
    assert!(
        markdown.contains("| ID | uk | ukr | ukr status | English | Sanskrit | primary |"),
        "human function table must expose exactly uk and ukr Ukrainian columns"
    );
    assert!(
        markdown.contains("| `1045` | текст-порожній? | порожній-текст? | stable | string-empty? |"),
        "human row 1045 must show uk followed by full Ukrainian ukr"
    );
    assert!(
        !markdown.contains("Повна українська"),
        "human table must not duplicate ukr under a second full-Ukrainian column"
    );
}
