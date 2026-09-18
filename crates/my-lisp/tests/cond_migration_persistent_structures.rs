//! P0 #613 migration guard for persistent structures.
//!
//! This is a source-shape/mechanism witness, not a second semantic authority:
//! behavioral meaning remains in the Lisp libraries and their existing
//! witnesses. The guard only proves that exact-Q numeric decisions no longer
//! enter historical two-part COND in this file family.

use my_lisp::{parse, Expr, ExprKind};

fn symbol(expr: &Expr) -> Option<&str> {
    match &expr.kind {
        ExprKind::Symbol(value) => Some(value.as_ref()),
        _ => None,
    }
}

fn is_numeric_comparison(expr: &Expr) -> bool {
    let ExprKind::List(items) = &expr.kind else {
        return false;
    };
    matches!(
        items.first().and_then(symbol),
        Some("<" | ">" | "<=" | ">=" | "=")
    )
}

fn count_two_part_numeric_cond_clauses(expr: &Expr) -> usize {
    match &expr.kind {
        ExprKind::List(items) => {
            // Quoted data is not executable control flow.
            if items.first().and_then(symbol) == Some("quote") {
                return 0;
            }

            let local = if items.first().and_then(symbol) == Some("cond") {
                items
                    .iter()
                    .skip(1)
                    .filter(|clause| {
                        let ExprKind::List(parts) = &clause.kind else {
                            return false;
                        };
                        parts.len() == 2 && is_numeric_comparison(&parts[0])
                    })
                    .count()
            } else {
                0
            };

            local
                + items
                    .iter()
                    .map(count_two_part_numeric_cond_clauses)
                    .sum::<usize>()
        }
        ExprKind::Pair(head, tail) => {
            count_two_part_numeric_cond_clauses(head)
                + count_two_part_numeric_cond_clauses(tail)
        }
        _ => 0,
    }
}

fn assert_no_two_part_numeric_cond(source_name: &str, source: &str) {
    let expressions = parse(source).expect("admitted library source must parse");
    let count: usize = expressions
        .iter()
        .map(count_two_part_numeric_cond_clauses)
        .sum();

    assert_eq!(
        count, 0,
        "{source_name} still has {count} two-part COND clause(s) fed by exact-Q numeric comparisons"
    );
}

#[test]
fn persistent_map_has_no_exact_q_two_part_cond_consumers() {
    assert_no_two_part_numeric_cond(
        "lib/persistent-map.lisp",
        include_str!("../../../lib/persistent-map.lisp"),
    );
}

#[test]
fn persistent_vector_has_no_exact_q_two_part_cond_consumers() {
    assert_no_two_part_numeric_cond(
        "lib/persistent-vector.lisp",
        include_str!("../../../lib/persistent-vector.lisp"),
    );
}
