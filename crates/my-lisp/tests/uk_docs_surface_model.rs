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
