use my_lisp::{eval_program, load_core_library, Session, Value};
use std::rc::Rc;

const COVERAGE: &str = include_str!("../../../lib/surface/uk-sa-coverage.wsm");
const UK_ACCEPTANCE: &str = include_str!("../../../lib/surface/uk-acceptance.my");

fn stable_uk_pairs() -> Vec<(&'static str, &'static str, &'static str)> {
    COVERAGE
        .lines()
        .filter_map(|line| {
            let fields = line.split_whitespace().collect::<Vec<_>>();
            if fields.first() != Some(&"(entry") || fields.get(5) != Some(&"stable") {
                return None;
            }
            Some((
                fields[1].trim_start_matches('('),
                fields[2],
                fields[3],
            ))
        })
        .collect()
}

fn uk_statuses() -> Vec<&'static str> {
    COVERAGE
        .lines()
        .filter_map(|line| {
            let fields = line.split_whitespace().collect::<Vec<_>>();
            (fields.first() == Some(&"(entry")).then(|| fields[5])
        })
        .collect()
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
        eval_program(source, &mut session).expect("surface prerequisite should load");
    }
    eval_program(include_str!("../../../lib/surface/uk.my"), &mut session)
        .expect("Ukrainian surface should load");
    session
}

fn is_same_runtime_value(left: &Value, right: &Value) -> bool {
    match (left, right) {
        // A builtin is an operation handle. Canon EN/UK/SA spellings and
        // ordinary aliases must retain one allocation, not merely similar code.
        (Value::Builtin(left), Value::Builtin(right)) => Rc::ptr_eq(left, right),
        _ => left == right,
    }
}

#[test]
fn every_stable_uk_surface_entry_resolves_to_its_declared_operation() {
    let pairs = stable_uk_pairs();
    assert_eq!(pairs.len(), 140, "coverage summary and entries drifted");

    let mut session = uk_session();
    let syntax = [
        ("quote", "як-є"),
        ("cond", "за-умовою"),
        ("lambda", "функція"),
        ("define", "визначити"),
    ];
    let mut checked_values = 0;

    for (category, english, ukrainian) in pairs {
        if syntax.contains(&(english, ukrainian)) {
            continue;
        }
        let english_value = eval_program(english, &mut session)
            .unwrap_or_else(|error| panic!("stable English value is missing: {category}/{english}: {error}"))
            .value;
        let ukrainian_value = eval_program(ukrainian, &mut session)
            .unwrap_or_else(|error| panic!("stable Ukrainian value is missing: {category}/{ukrainian}: {error}"))
            .value;
        assert!(
            is_same_runtime_value(&english_value, &ukrainian_value),
            "stable surface changed runtime identity: {category}/{english} -> {ukrainian}"
        );
        checked_values += 1;
    }

    // Four evaluation-control/necessary forms are verified behaviorally in
    // uk_surface.rs and uk_sa_surface.rs; all other stable rows resolve to
    // first-class values. Canon values are resolver-owned, not mutable aliases.
    assert_eq!(checked_values, 136);
}

#[test]
fn selected_ukrainian_surface_has_no_candidate_or_missing_rows() {
    let statuses = uk_statuses();
    let stable = statuses.iter().filter(|status| **status == "stable").count();
    let compatibility = statuses
        .iter()
        .filter(|status| **status == "compatibility-only")
        .count();

    assert_eq!(statuses.len(), 161);
    assert_eq!(stable, 140);
    assert_eq!(compatibility, 21);
    assert_eq!(stable + compatibility, statuses.len());
}

fn is_ukrainian_layout_identifier_char(character: char) -> bool {
    "абвгґдеєжзиіїйклмнопрстуфхцчшщьюяАБВГҐДЕЄЖЗИІЇЙКЛМНОПРСТУФХЦЧШЩЬЮЯ0123456789-?!'*"
        .contains(character)
}

#[test]
fn every_stable_ukrainian_name_is_typeable_on_the_ukrainian_layout() {
    for (category, _english, ukrainian) in stable_uk_pairs() {
        assert!(
            ukrainian.chars().all(is_ukrainian_layout_identifier_char),
            "stable Ukrainian name needs another keyboard layout: {category}/{ukrainian}"
        );
    }
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

#[test]
fn ukrainian_acceptance_program_code_never_requires_latin_layout() {
    let code = executable_characters(UK_ACCEPTANCE);
    let latin = code
        .chars()
        .filter(|character| character.is_ascii_alphabetic())
        .collect::<String>();
    assert!(
        latin.is_empty(),
        "executable Ukrainian program still contains Latin letters: {latin}"
    );
}
