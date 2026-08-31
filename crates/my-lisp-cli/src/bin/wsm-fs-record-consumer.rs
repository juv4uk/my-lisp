//! Consume the bounded F6 record stream and reconstruct its WSM root.
//! Stored envelopes are quoted data; this binary never evaluates their payload.

use my_lisp::{eval_program, Environment, Session, Value};
use std::io::{self, Read};

fn string_literal(value: &str) -> String {
    format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\""))
}

fn main() {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).unwrap();
    let records: Vec<_> = input
        .lines()
        .filter(|line| !line.trim().is_empty())
        .collect();
    if records.len() != 2 {
        eprintln!("expected exactly two records");
        std::process::exit(2);
    }
    let mut session = Session {
        environment: Environment::root(),
    };
    for source in [
        include_str!("../../../../lib/core.my"),
        include_str!("../../../../lib/unify.my"),
        include_str!("../../../../lib/reason.my"),
        include_str!("../../../../lib/forward.my"),
        include_str!("../../../../lib/knowledge.my"),
        include_str!("../../../../lib/persistent-map.my"),
        include_str!("../../../../lib/world.my"),
        include_str!("../../../../lib/content-store.my"),
        include_str!("../../../../lib/lisp-fs.my"),
    ] {
        eval_program(source, &mut session).unwrap();
    }
    let root = eval_program(
        &format!("(fs-deserialize-root {})", string_literal(records[0])),
        &mut session,
    )
    .unwrap()
    .value;
    let _object = eval_program(
        &format!("(fs-deserialize-object {})", string_literal(records[1])),
        &mut session,
    )
    .unwrap()
    .value;
    let Value::Pair(root_tag, _) = &root else {
        std::process::exit(1)
    };
    let Value::Symbol(tag) = &**root_tag else {
        std::process::exit(1)
    };
    if &**tag != "accepted" {
        std::process::exit(1);
    }
    let expr = format!(
        "(fs-reconstruct-root (quote {}) (list (quote {})))",
        records[0], records[1]
    );
    let rebuilt = eval_program(&expr, &mut session).unwrap().value;
    let Value::Pair(ref tag, _) = rebuilt else {
        std::process::exit(1)
    };
    if !matches!(&**tag, Value::Symbol(name) if &**name == "accepted") {
        std::process::exit(1);
    }
    println!("f6-semantic-consumer-ok records=2");
}
