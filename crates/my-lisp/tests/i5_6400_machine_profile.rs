use std::path::Path;

#[test]
fn i5_6400_profile_projects_existing_semantic_identities() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let profile_path = root.join("lib/machine/intel-core-i5-6400.lisp");
    let profile = std::fs::read_to_string(&profile_path)
        .unwrap_or_else(|error| panic!("missing i5-6400 machine profile at {profile_path:?}: {error}"));

    assert!(profile.contains("(machine-profile/1"));
    assert!(profile.contains("(cpu intel-core-i5-6400)"));
    assert!(profile.contains("(microarchitecture skylake)"));
    assert!(profile.contains("(isa x86-64)"));

    for (semantic_id, expected_machine_path) in [
        ("0002", "tag-test"),
        ("0003", "CMP/SETE"),
        ("0005", "LOAD-pair-head"),
        ("0006", "LOAD-pair-tail"),
        ("0104", "ADD"),
        ("1001", "SUB/NEG"),
        ("1002", "IMUL"),
        ("1007", "IDIV-remainder"),
        ("1008", "IDIV-quotient"),
        ("1014", "CMP/SETL"),
        ("1015", "CMP/SETG"),
        ("1016", "CMP/SETE"),
        ("1066", "LOAD-vector-length"),
        ("1067", "LOAD-vector-element"),
        ("1068", "STORE-vector-element"),
        ("1072", "LOAD-buffer-length"),
        ("1073", "LOAD-buffer-element"),
    ] {
        let row_prefix = format!("({semantic_id} ");
        let row = profile
            .lines()
            .map(str::trim_start)
            .find(|line| line.starts_with(&row_prefix))
            .unwrap_or_else(|| panic!("i5-6400 profile missing semantic ID {semantic_id}"));
        assert!(
            row.contains(expected_machine_path),
            "semantic ID {semantic_id} must advertise {expected_machine_path:?}; row: {row}"
        );
    }
}

#[test]
fn generated_function_table_has_i5_6400_column_without_replacing_semantic_authority() {
    let markdown = include_str!("../../../docs/generated/function-table.md");
    assert!(
        markdown.contains("Intel Core i5-6400 / Skylake"),
        "human function table must expose the requested processor column"
    );
    assert!(
        markdown.contains("| `0104` | додати | додати | stable | — | yoga | stable | ADD"),
        "semantic ID 0104 must show the i5-6400 ADD fast path"
    );
    assert!(
        markdown.contains("| `0005` | перше | перше | stable | car | ādi | stable | LOAD-pair-head"),
        "Canon CAR identity must show its direct memory-load realization"
    );

    let semantic_table = include_str!("../../../lib/generated/function-table.lisp");
    assert!(
        !semantic_table.contains("intel-core-i5-6400"),
        "processor-specific realization must not contaminate the semantic machine-readable function table"
    );
}
