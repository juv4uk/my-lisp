//! Documentation/governance/policy checks relocated out of `cargo test`.
//! Перевірки документації/врядування/політик, перенесені з `cargo test`.

use std::process::Command;

pub struct Check {
    pub name: &'static str,
    pub run: fn() -> Result<(), String>,
}

pub fn all() -> Vec<Check> {
    vec![
        Check {
            name: "meta-eval-evidence-matrix",
            run: meta_eval_evidence_matrix,
        },
        Check {
            name: "meta-eval-human-evidence-projection",
            run: meta_eval_human_evidence_projection,
        },
        Check {
            name: "semantic-ownership-map-in-sync",
            run: semantic_ownership_map_in_sync,
        },
        Check {
            name: "public-docs-share-current-project-identity-and-extension",
            run: public_docs_share_current_project_identity_and_extension,
        },
        Check {
            name: "public-docs-point-to-semantic-authority",
            run: public_docs_point_to_semantic_authority,
        },
        Check {
            name: "host-semantic-surface-documentation-tracks-time-ownership",
            run: host_semantic_surface_documentation_tracks_time_ownership,
        },
        Check {
            name: "agent-onboarding-records-removed-coordination-surface",
            run: agent_onboarding_records_removed_coordination_surface,
        },
        Check {
            name: "current-agent-authority-records-removed-legacy-coordination",
            run: current_agent_authority_records_removed_legacy_coordination,
        },
        Check {
            name: "human-migration-doc-keeps-semantic-and-coordination-planes-separate",
            run: human_migration_doc_keeps_semantic_and_coordination_planes_separate,
        },
        Check {
            name: "s2-explicitly-contracts-category-not-error-wording",
            run: s2_explicitly_contracts_category_not_error_wording,
        },
        Check {
            name: "vsi-stable-ukrainski-nazvy-maiut-numeric-zapys-u-dovidnyku",
            run: vsi_stable_ukrainski_nazvy_maiut_numeric_zapys_u_dovidnyku,
        },
        Check {
            name: "dokumentatsiinyi-kliuch-ie-tilky-numeric",
            run: dokumentatsiinyi_kliuch_ie_tilky_numeric,
        },
        Check {
            name: "znak-pytannia-tochno-vidpovidaie-predykatam",
            run: znak_pytannia_tochno_vidpovidaie_predykatam,
        },
        Check {
            name: "znak-oklyku-tochno-vidpovidaie-mutatsii",
            run: znak_oklyku_tochno_vidpovidaie_mutatsii,
        },
        Check {
            name: "smyslovyi-audyt-summary-zbihaietsia-z-faktychnymy-danymy",
            run: smyslovyi_audyt_summary_zbihaietsia_z_faktychnymy_danymy,
        },
        Check {
            name: "seredovyshche-ne-maie-povtornoho-surface-binding",
            run: seredovyshche_ne_maie_povtornoho_surface_binding,
        },
        Check {
            name: "stari-nazvy-smystovoho-audytu-lyshaiutsia-aliasamy-sumisnosti",
            run: stari_nazvy_smystovoho_audytu_lyshaiutsia_aliasamy_sumisnosti,
        },
        Check {
            name: "every-stable-ukrainian-name-is-typeable-on-the-ukrainian-layout",
            run: every_stable_ukrainian_name_is_typeable_on_the_ukrainian_layout,
        },
        Check {
            name: "ukrainian-acceptance-program-code-never-requires-latin-layout",
            run: ukrainian_acceptance_program_code_never_requires_latin_layout,
        },
        Check {
            name: "ukrainska-prohrama-pryinnyattia-ne-potrebuie-latynskoi-rozkladky",
            run: ukrainska_prohrama_pryinnyattia_ne_potrebuie_latynskoi_rozkladky,
        },
    ]
}

fn run_python(script: &str, args: &[&str], label: &str) -> Result<(), String> {
    let output = Command::new("python3")
        .arg(script)
        .args(args)
        .output()
        .map_err(|error| format!("python3 must run {label}: {error}"))?;

    if output.status.success() {
        Ok(())
    } else {
        Err(format!(
            "{label} failed\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ))
    }
}

// --- ported from crates/my-lisp/tests/meta_eval_evidence_matrix.rs ---

fn meta_eval_evidence_matrix() -> Result<(), String> {
    let script = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../scripts/check-meta-eval-evidence.py"
    );
    run_python(script, &[], "meta-eval evidence checker")
}

fn meta_eval_human_evidence_projection() -> Result<(), String> {
    let script = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../scripts/generate-meta-eval-evidence.py"
    );
    run_python(
        script,
        &["--check"],
        "meta-eval human evidence projection check",
    )
}

// --- ported from crates/my-lisp/tests/semantic_ownership.rs ---

fn semantic_ownership_map_in_sync() -> Result<(), String> {
    let script = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../scripts/semantic-ownership.py"
    );
    run_python(script, &["--check"], "semantic ownership checker")
}

// --- ported from crates/my-lisp/tests/documentation_contract.rs ---

fn public_docs_share_current_project_identity_and_extension() -> Result<(), String> {
    let readme = include_str!("../../../README.md");
    let core = include_str!("../../../docs/language-core.md");

    for (doc_name, doc) in [("README.md", readme), ("docs/language-core.md", core)] {
        if !(doc.contains("reference implementation") || doc.contains("референсна реалізація")) {
            return Err(format!(
                "{doc_name}: public architecture prose must describe Rust as a reference implementation"
            ));
        }
        if doc.contains("canonical Rust implementation")
            || doc.contains("канонічна реалізація на Rust")
            || doc.contains("kanonische Rust-Implementierung")
        {
            return Err(format!(
                "{doc_name}: implementation wording must not imply that Rust itself owns semantics"
            ));
        }
        // Not just "all three extension tokens are mentioned somewhere" (that
        // passed while README and language-core.md contradicted each other
        // on which extension is canonical) — the doc must actually state
        // `.lisp` as canonical and the other two as legacy aliases.
        let states_lisp_canonical = doc.contains("канонічне розширення вихідного коду — **`.lisp`**")
            || doc.contains("Канонічне розширення вихідного коду — **`.lisp`**")
            || doc.contains("canonical source extension is **`.lisp`**")
            || doc.contains("current canonical source extension is **`.lisp`**");
        let states_others_legacy_aliases = (doc.contains("`.wsm`") && doc.contains("`.my`"))
            && (doc.contains("legacy alias") || doc.contains("legacy aliases"));
        if !states_lisp_canonical || !states_others_legacy_aliases {
            return Err(format!(
                "{doc_name}: public architecture prose must state `.lisp` as the canonical \
                 extension and `.wsm`/`.my` as legacy aliases (per my-lisp#81)"
            ));
        }
    }
    Ok(())
}

fn public_docs_point_to_semantic_authority() -> Result<(), String> {
    let readme = include_str!("../../../README.md");
    let core = include_str!("../../../docs/language-core.md");
    let authority = include_str!("../../../docs/semantic-authority-map.md");
    let authority_lower = authority.to_lowercase();

    let mut problems = Vec::new();
    if !readme.contains("docs/semantic-authority-map.md") {
        problems.push("README.md must link docs/semantic-authority-map.md");
    }
    if !core.contains("semantic-authority-map.md") {
        problems.push("docs/language-core.md must link semantic-authority-map.md");
    }
    if !authority.contains("language-contract.my") {
        problems.push("docs/semantic-authority-map.md must reference language-contract.my");
    }
    if !authority_lower.contains("ratified adr") {
        problems.push("docs/semantic-authority-map.md must mention ratified ADRs");
    }
    if !authority_lower.contains("executable conformance") {
        problems.push("docs/semantic-authority-map.md must mention executable conformance");
    }
    if problems.is_empty() {
        Ok(())
    } else {
        Err(problems.join("; "))
    }
}

fn host_semantic_surface_documentation_tracks_time_ownership() -> Result<(), String> {
    let hss = include_str!("../../../docs/host-semantic-surface.md");
    let time = include_str!("../../../lib/time.my");
    let builtins = include_str!("../../my-lisp/src/eval/builtins.rs");

    let mut problems = Vec::new();
    if !hss.contains("mono-ns") {
        problems.push("host-semantic-surface.md missing mono-ns".to_string());
    }
    if !hss.contains("unix-time-now") {
        problems.push("host-semantic-surface.md missing unix-time-now".to_string());
    }
    if !hss.contains("`utc-now` | `lib/time.my` | derived public clock meaning | HOST REMOVED") {
        problems.push("host-semantic-surface.md missing utc-now HOST REMOVED row".to_string());
    }
    if !time.contains("(def mono-ms") {
        problems.push("lib/time.my missing (def mono-ms".to_string());
    }
    if !time.contains("(def utc-now") {
        problems.push("lib/time.my missing (def utc-now".to_string());
    }
    if builtins.contains("fn civil_from_days")
        || builtins.contains("fn utc_now_value")
        || builtins.contains("\"utc-now\",")
    {
        problems.push(
            "Rust must not regain Gregorian utc-now semantics after the completed migration"
                .to_string(),
        );
    }
    if !builtins.contains("\"unix-time-now\",") {
        problems.push("builtins.rs missing \"unix-time-now\",".to_string());
    }
    if problems.is_empty() {
        Ok(())
    } else {
        Err(problems.join("; "))
    }
}

fn agent_onboarding_records_removed_coordination_surface() -> Result<(), String> {
    let agents = include_str!("../../../AGENTS.md");
    let deprecation = include_str!("../../../knowledge/swarm-legacy-deprecation.wsm");

    let mut problems = Vec::new();
    for needle in [
        "my-lisp :9999",
        "swarm-node :910x",
        "Стара coordination surface на `:9999` фізично видалена",
        "мають повертати `unknown op`",
        "knowledge/swarm-legacy-deprecation.wsm",
    ] {
        if !agents.contains(needle) {
            problems.push(format!("AGENTS.md missing: {needle}"));
        }
    }
    if agents.contains("This is a\n  first-class pattern, not a fallback") {
        problems
            .push("legacy :9999 mailbox instructions must not return as current onboarding".to_string());
    }
    for needle in [
        "(status . deprecated)",
        "(physical-status . removed)",
        "(runtime-rejection . confirmed)",
        "(coordination-authority . swarm-node)",
    ] {
        if !deprecation.contains(needle) {
            problems.push(format!("swarm-legacy-deprecation.wsm missing: {needle}"));
        }
    }
    if problems.is_empty() {
        Ok(())
    } else {
        Err(problems.join("; "))
    }
}

// --- ported from crates/my-lisp/tests/swarm_deprecation.rs ---

fn current_agent_authority_records_removed_legacy_coordination() -> Result<(), String> {
    let agent_guide = include_str!("../../../AGENTS.md");
    let mut problems = Vec::new();
    for needle in [
        "Current coordination authority:",
        "`swarm-node`",
        "Стара coordination surface на `:9999` фізично видалена",
        "мають повертати `unknown op`",
        "my-lisp :9999",
        "swarm-node :910x",
    ] {
        if !agent_guide.contains(needle) {
            problems.push(format!("AGENTS.md missing: {needle}"));
        }
    }
    if problems.is_empty() {
        Ok(())
    } else {
        Err(problems.join("; "))
    }
}

fn human_migration_doc_keeps_semantic_and_coordination_planes_separate() -> Result<(), String> {
    let mesh_doc = include_str!("../../../docs/swarm-mesh-v2.md");
    let mut problems = Vec::new();
    for needle in [
        "my-lisp :9999",
        "swarm-node :910x",
        "no longer the\ncoordination path going forward",
        "semantic oracle",
    ] {
        if !mesh_doc.contains(needle) {
            problems.push(format!("docs/swarm-mesh-v2.md missing: {needle}"));
        }
    }
    if problems.is_empty() {
        Ok(())
    } else {
        Err(problems.join("; "))
    }
}

// --- ported from crates/my-lisp/tests/meta_eval_error_detail_boundary.rs ---

fn s2_explicitly_contracts_category_not_error_wording() -> Result<(), String> {
    let axioms = include_str!("../../../docs/language-core-axioms.md");
    if !axioms.contains("The wording may differ; the *category* is the contract.") {
        return Err("S2 must state the error-detail boundary explicitly".to_string());
    }

    let error_source = include_str!("../../my-lisp/src/error.rs");
    if !error_source.contains("non-contractual: `kind` is what S2 ratifies") {
        return Err(
            "the reference error type must keep the contractual axis explicit".to_string(),
        );
    }
    Ok(())
}

// --- ported from crates/my-lisp/tests/ukrainian_api_docs.rs ---

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

fn documented() -> Result<BTreeMap<(String, String), String>, String> {
    let mut result = BTreeMap::new();
    for line in DOCS_INDEX.lines() {
        let fields = line.split_whitespace().collect::<Vec<_>>();
        if fields.first() != Some(&"(doc") {
            continue;
        }
        if fields.len() < 5 {
            return Err(format!(
                "рядок документації має містити category numeric-ID UK kind: {line}"
            ));
        }
        result.insert(
            (fields[2].to_string(), fields[3].to_string()),
            fields[4].to_string(),
        );
    }
    Ok(result)
}

fn vsi_stable_ukrainski_nazvy_maiut_numeric_zapys_u_dovidnyku() -> Result<(), String> {
    let coverage = stable_pairs();
    let docs = documented()?;

    // Floor, not exact count: the registry only grows, so restating an exact
    // literal here would silently rot (TEST-ARCHITECTURE-1 step 2). The real
    // invariant is the set-equality check below; this floor only guards
    // against both sides degenerating to an (agreeing) near-empty set.
    if coverage.len() <= 50 {
        return Err(format!(
            "expected a substantial number of stable UK-covered names, found {}",
            coverage.len()
        ));
    }
    if coverage != docs.keys().cloned().collect::<BTreeSet<_>>() {
        return Err("numeric registry і український документаційний індекс розійшлися".to_string());
    }
    for (_, uk) in &coverage {
        if !DOCS_MD.contains(&format!("| `{uk}` |")) {
            return Err(format!(
                "публічне українське ім'я відсутнє у Markdown-довіднику: {uk}"
            ));
        }
    }
    Ok(())
}

fn dokumentatsiinyi_kliuch_ie_tilky_numeric() -> Result<(), String> {
    for ((identity, _uk), _) in documented()? {
        if !(identity.len() >= 4 && identity.chars().all(|ch| ch.is_ascii_digit())) {
            return Err(format!(
                "документаційний join key не може бути EN spelling: {identity}"
            ));
        }
    }
    Ok(())
}

fn znak_pytannia_tochno_vidpovidaie_predykatam() -> Result<(), String> {
    let docs = documented()?;
    let mut predicates = 0usize;
    for ((_id, uk), kind) in docs {
        let question_name = uk.ends_with('?');
        let predicate = kind == "predicate";
        if question_name != predicate {
            return Err(format!(
                "українська назва {uk}: знак ? і predicate мусять збігатися"
            ));
        }
        predicates += usize::from(predicate);
    }
    // Floor, not exact count: derived only as an anti-vacuousness guard so
    // this loop can't silently pass by iterating zero predicates.
    if predicates <= 10 {
        return Err(format!("публічний каталог предикатів схлопнувся: {predicates}"));
    }
    Ok(())
}

fn znak_oklyku_tochno_vidpovidaie_mutatsii() -> Result<(), String> {
    let docs = documented()?;
    let mut mutations = 0usize;
    for ((_id, uk), kind) in docs {
        let mutation = kind == "mutation";
        if uk.ends_with('!') != mutation {
            return Err(format!("{uk}: ! зарезервований для мутації"));
        }
        if mutation {
            mutations += 1;
            if uk != "встановити-елемент-вектора!" {
                return Err(format!("несподівана мутація: {uk}"));
            }
        }
    }
    // Floor, not exact count: today's single known mutation name is checked
    // above by exact value; this only guards against the mutation-kind loop
    // being silently skipped entirely.
    if mutations < 1 {
        return Err("публічний каталог мутацій схлопнувся".to_string());
    }
    Ok(())
}

/// Parse `(tag N)` out of NAME_AUDIT's summary header, e.g.
/// `(stable-reviewed 140)` -> `("stable-reviewed", 140)`.
fn name_audit_summary_count(tag: &str) -> Result<usize, String> {
    NAME_AUDIT
        .lines()
        .find_map(|line| {
            let line = line.trim();
            let rest = line.strip_prefix(&format!("({tag} "))?;
            rest.trim_end_matches(')').parse::<usize>().ok()
        })
        .ok_or_else(|| format!("NAME_AUDIT missing ({tag} N) summary line"))
}

fn smyslovyi_audyt_summary_zbihaietsia_z_faktychnymy_danymy() -> Result<(), String> {
    let reviewed = name_audit_summary_count("stable-reviewed")?;
    let renamed = name_audit_summary_count("renamed")?;
    let retained = name_audit_summary_count("retained")?;
    let actual_rename_lines = NAME_AUDIT
        .lines()
        .filter(|line| line.trim_start().starts_with("(rename "))
        .count();

    // Derived, not restated: the summary header must agree with the actual
    // data below it, and with the registry's own stable-UK count -- this
    // catches drift between the two without hardcoding either number twice
    // (TEST-ARCHITECTURE-1 step 2).
    if reviewed != renamed + retained {
        return Err(format!(
            "stable-reviewed ({reviewed}) мусить дорівнювати renamed+retained ({renamed}+{retained})"
        ));
    }
    if renamed != actual_rename_lines {
        return Err(format!(
            "(renamed {renamed}) розійшовся з фактичною кількістю (rename ...) рядків: {actual_rename_lines}"
        ));
    }
    let stable = stable_pairs().len();
    if reviewed != stable {
        return Err(format!(
            "smyslovyi audit ({reviewed}) мусить покривати рівно stable UK-покриття реєстру ({stable})"
        ));
    }
    Ok(())
}

fn seredovyshche_ne_maie_povtornoho_surface_binding() -> Result<(), String> {
    let needle = "(define середовище env)";
    let count = UK_SURFACE.match_indices(needle).count();
    if count != 1 {
        Err(format!("expected exactly 1 occurrence of {needle}, found {count}"))
    } else {
        Ok(())
    }
}

fn stari_nazvy_smystovoho_audytu_lyshaiutsia_aliasamy_sumisnosti() -> Result<(), String> {
    for line in NAME_AUDIT.lines() {
        let fields = line.split_whitespace().collect::<Vec<_>>();
        if fields.first() != Some(&"(rename") {
            continue;
        }
        let en = fields[1];
        let old = fields[2];
        let new = fields[3];
        if !UK_SURFACE.contains(&format!("(define {old} {en})")) {
            return Err(format!("missing alias (define {old} {en})"));
        }
        if !UK_SURFACE.contains(&format!("(define {new} {en})")) {
            return Err(format!("missing alias (define {new} {en})"));
        }
    }
    Ok(())
}

// --- ported from crates/my-lisp/tests/uk_surface_equivalence.rs and
// crates/my-lisp/tests/ukrainska_programa_pryimannya.rs: keyboard/text-policy
// lints, not semantic mutation tests (TEST-ARCHITECTURE-1 step 4). ---

const UK_ACCEPTANCE: &str = include_str!("../../../lib/surface/uk-acceptance.my");
const RIVNOPRAVNIST_UK: &str = include_str!("../../../tests/fixtures/rivnopravnist-uk.my");

#[derive(PartialEq, Eq)]
enum SurfaceAdmission {
    Stable,
    Other,
}

/// Every semantic ID whose EN spelling AND UK spelling are both `stable` --
/// mirrors `uk_surface_equivalence.rs::stable_en_uk_pairs` (crate-integration
/// test, not reachable from here), kept in sync by hand since this and that
/// file read the same `semantic-registry.wsm` but serve different purposes
/// (behavior vs. keyboard-layout lint).
fn stable_en_uk_names_needing_uk_layout_check() -> Vec<String> {
    REGISTRY
        .lines()
        .filter_map(|line| {
            let fields = line.split_whitespace().collect::<Vec<_>>();
            let semantic_id = fields.first()?.strip_prefix('(')?;
            if semantic_id.is_empty() || !semantic_id.bytes().all(|b| b.is_ascii_digit()) {
                return None;
            }
            let mut en = None;
            let mut uk = None;
            for triple in fields[1..].chunks(3) {
                if triple.len() != 3 {
                    break;
                }
                let namespace = triple[0].trim_start_matches('(');
                let name = triple[1];
                let admission = if triple[2].trim_end_matches(')') == "stable" {
                    SurfaceAdmission::Stable
                } else {
                    SurfaceAdmission::Other
                };
                if admission == SurfaceAdmission::Stable {
                    match namespace {
                        "en" => en = Some(name.to_string()),
                        "uk" => uk = Some(name.to_string()),
                        _ => {}
                    }
                }
            }
            match (en, uk) {
                (Some(_en), Some(uk)) => Some(uk),
                _ => None,
            }
        })
        .collect()
}

fn is_ukrainian_layout_identifier_char(character: char) -> bool {
    "абвгґдеєжзиіїйклмнопрстуфхцчшщьюяАБВГҐДЕЄЖЗИІЇЙКЛМНОПРСТУФХЦЧШЩЬЮЯ0123456789-?!'*"
        .contains(character)
}

fn every_stable_ukrainian_name_is_typeable_on_the_ukrainian_layout() -> Result<(), String> {
    for ukrainian in stable_en_uk_names_needing_uk_layout_check() {
        if !ukrainian.chars().all(is_ukrainian_layout_identifier_char) {
            return Err(format!(
                "stable Ukrainian name needs another keyboard layout: {ukrainian}"
            ));
        }
    }
    Ok(())
}

fn executable_characters(source: &str) -> String {
    let mut output = String::new();
    let mut characters = source.chars().peekable();
    let mut in_string = false;

    while let Some(character) = characters.next() {
        if in_string {
            match character {
                '\\' => {
                    characters.next();
                }
                '"' => in_string = false,
                _ => {}
            }
        } else {
            match character {
                ';' => {
                    for comment_character in characters.by_ref() {
                        if comment_character == '\n' {
                            output.push('\n');
                            break;
                        }
                    }
                }
                '"' => in_string = true,
                _ => output.push(character),
            }
        }
    }
    output
}

fn ukrainian_acceptance_program_code_never_requires_latin_layout() -> Result<(), String> {
    let code = executable_characters(UK_ACCEPTANCE);
    let latin = code
        .chars()
        .filter(|character| character.is_ascii_alphabetic())
        .collect::<String>();
    if !latin.is_empty() {
        return Err(format!(
            "executable Ukrainian program still contains Latin letters: {latin}"
        ));
    }
    Ok(())
}

/// Strips `;`-to-end-of-line comments from each line (a simpler pass than
/// `executable_characters` above since this fixture has no string literals
/// containing `;`), mirroring
/// `ukrainska_programa_pryimannya.rs`'s `виконуваний_код`.
fn rivnopravnist_uk_executable_code() -> String {
    RIVNOPRAVNIST_UK
        .lines()
        .map(|line| line.split(';').next().unwrap_or_default())
        .collect::<Vec<_>>()
        .join("\n")
}

fn ukrainska_prohrama_pryinnyattia_ne_potrebuie_latynskoi_rozkladky() -> Result<(), String> {
    let code = rivnopravnist_uk_executable_code();
    let latin = code
        .chars()
        .filter(|character| character.is_ascii_alphabetic())
        .collect::<String>();
    if !latin.is_empty() {
        return Err(format!(
            "у виконуваному українському коді знайдено латинські літери: {latin:?}"
        ));
    }
    Ok(())
}
