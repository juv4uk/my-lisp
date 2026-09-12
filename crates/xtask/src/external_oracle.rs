//! Exact arithmetic AST projection for WSM-5's independent external oracle.
//!
//! This layer does not interpret arithmetic by spelling and does not evaluate
//! my-lisp. It parses one source expression, resolves the operator through the
//! authoritative semantic registry, and projects the exact AST shape into a
//! Wolfram Language expression. Unsupported input fails closed with stable
//! codes instead of being approximated.

use my_lisp::semantic_registry_export::semantic_id_for_admitted_surface;
use my_lisp::{Exactness, Expr, ExprKind};

#[derive(Debug, Clone, PartialEq, Eq)]
struct Unsupported {
    code: &'static str,
}

impl Unsupported {
    const fn new(code: &'static str) -> Self {
        Self { code }
    }
}

fn translate_source(source: &str) -> Result<String, Unsupported> {
    let expressions = my_lisp::parse(source)
        .map_err(|_| Unsupported::new("external-oracle/parse"))?;
    if expressions.len() != 1 {
        return Err(Unsupported::new("external-oracle/top-level-arity"));
    }
    translate_expr(&expressions[0])
}

fn translate_expr(expr: &Expr) -> Result<String, Unsupported> {
    match &expr.kind {
        ExprKind::Number(number, Exactness::Inexact) => {
            Err(Unsupported::new("external-oracle/inexact-number"))
        }
        ExprKind::Number(number, Exactness::Exact) => {
            // Reader-produced exact Number values are compact exact integers
            // in binary64's lossless integer range. Refuse any hand-built AST
            // that violates that representation invariant rather than printing
            // an approximate decimal.
            if number.is_finite()
                && number.fract() == 0.0
                && number.abs() <= 9_007_199_254_740_992.0
            {
                Ok(format!("{}", *number as i64))
            } else {
                Err(Unsupported::new(
                    "external-oracle/exact-number-representation",
                ))
            }
        }
        ExprKind::Rational(rational) => Ok(rational.to_string()),
        ExprKind::String(_) => Err(Unsupported::new("external-oracle/string")),
        ExprKind::Pair(_, _) => Err(Unsupported::new("external-oracle/pair")),
        ExprKind::NumericBuffer(_) => {
            Err(Unsupported::new("external-oracle/numeric-buffer"))
        }
        ExprKind::Symbol(_) => Err(Unsupported::new("external-oracle/bare-symbol")),
        ExprKind::List(items) => translate_call(items),
    }
}

fn translate_call(items: &[Expr]) -> Result<String, Unsupported> {
    let Some((head, arguments)) = items.split_first() else {
        return Err(Unsupported::new("external-oracle/empty-list"));
    };
    let ExprKind::Symbol(surface) = &head.kind else {
        return Err(Unsupported::new("external-oracle/non-symbol-head"));
    };

    let semantic_id = semantic_id_for_admitted_surface(surface)
        .ok_or_else(|| Unsupported::new("external-oracle/unknown-semantic-id"))?;

    let translated = arguments
        .iter()
        .map(translate_expr)
        .collect::<Result<Vec<_>, _>>()?;

    match semantic_id {
        "0104" => Ok(format!("Total[{{{}}}]", translated.join(", "))),
        "1001" => match translated.as_slice() {
            [] => Err(Unsupported::new("external-oracle/arity")),
            [only] => Ok(format!("Minus[{only}]")),
            _ => Ok(format!("Fold[Subtract, {{{}}}]", translated.join(", "))),
        },
        "1002" => Ok(format!("Times[{}]", translated.join(", "))),
        "1003" => match translated.as_slice() {
            [] => Err(Unsupported::new("external-oracle/arity")),
            [only] => Ok(format!("Divide[1, {only}]")),
            _ => Ok(format!("Fold[Divide, {{{}}}]", translated.join(", "))),
        },
        _ => Err(Unsupported::new("external-oracle/unknown-semantic-id")),
    }
}

#[cfg(test)]
mod tests {
    use super::{translate_expr, translate_source};
    use my_lisp::semantic_registry_export::admitted_surfaces_for_semantic_id;
    use my_lisp::{Exactness, Expr, ExprKind, NumericBuffer, Span};
    use std::rc::Rc;

    #[test]
    fn nary_division_preserves_left_fold() {
        assert_eq!(
            translate_source("(/ 5 6 8 7)").unwrap(),
            "Fold[Divide, {5, 6, 8, 7}]"
        );
    }

    #[test]
    fn nested_division_remains_structurally_distinct() {
        assert_eq!(
            translate_source("(/ (/ 5 6) (/ 8 7))").unwrap(),
            "Fold[Divide, {Fold[Divide, {5, 6}], Fold[Divide, {8, 7}]}]"
        );
    }

    #[test]
    fn addition_uses_total_and_exact_zero_identity() {
        assert_eq!(
            translate_source("(+ 1 2 3)").unwrap(),
            "Total[{1, 2, 3}]"
        );
        assert_eq!(translate_source("(+)").unwrap(), "Total[{}]");
    }

    #[test]
    fn multiplication_uses_times_and_exact_one_identity() {
        assert_eq!(translate_source("(* 2 3 4)").unwrap(), "Times[2, 3, 4]");
        assert_eq!(translate_source("(*)").unwrap(), "Times[]");
    }

    #[test]
    fn unary_subtraction_and_reciprocal_are_explicit() {
        assert_eq!(translate_source("(- 3)").unwrap(), "Minus[3]");
        assert_eq!(translate_source("(/ 4)").unwrap(), "Divide[1, 4]");
    }

    #[test]
    fn all_admitted_arithmetic_surfaces_project_by_semantic_identity() {
        let cases = [
            ("0104", "Total[{8, 2}]"),
            ("1001", "Fold[Subtract, {8, 2}]"),
            ("1002", "Times[8, 2]"),
            ("1003", "Fold[Divide, {8, 2}]"),
        ];

        for (semantic_id, expected) in cases {
            let surfaces = admitted_surfaces_for_semantic_id(semantic_id);
            assert!(
                !surfaces.is_empty(),
                "semantic identity {semantic_id} must have admitted surfaces"
            );
            for surface in surfaces {
                let source = format!("({} 8 2)", surface.name);
                assert_eq!(
                    translate_source(&source).unwrap(),
                    expected,
                    "surface {} in namespace {} must project through semantic identity {}",
                    surface.name,
                    surface.namespace,
                    semantic_id
                );
            }
        }
    }

    #[test]
    fn source_must_contain_exactly_one_top_level_expression() {
        let error = translate_source("(+ 1 2) (* 3 4)").unwrap_err();
        assert_eq!(error.code, "external-oracle/top-level-arity");
    }

    #[test]
    fn unsupported_ast_kinds_fail_closed_with_stable_codes() {
        let span = Span::default();
        let exact_one = || Expr {
            kind: ExprKind::Number(1.0, Exactness::Exact),
            span,
        };

        let cases = [
            (
                Expr {
                    kind: ExprKind::Number(0.5, Exactness::Inexact),
                    span,
                },
                "external-oracle/inexact-number",
            ),
            (
                Expr {
                    kind: ExprKind::String(Rc::from("text")),
                    span,
                },
                "external-oracle/string",
            ),
            (
                Expr {
                    kind: ExprKind::Pair(Rc::new(exact_one()), Rc::new(exact_one())),
                    span,
                },
                "external-oracle/pair",
            ),
            (
                Expr {
                    kind: ExprKind::NumericBuffer(NumericBuffer::I32(vec![1].into())),
                    span,
                },
                "external-oracle/numeric-buffer",
            ),
            (
                Expr {
                    kind: ExprKind::Symbol(Rc::from("x")),
                    span,
                },
                "external-oracle/bare-symbol",
            ),
        ];

        for (expr, expected_code) in cases {
            let error = translate_expr(&expr).unwrap_err();
            assert_eq!(error.code, expected_code);
        }
    }

    #[test]
    fn unknown_operation_fails_closed_instead_of_guessing_by_spelling() {
        let error = translate_source("(mystery-op 1 2)").unwrap_err();
        assert_eq!(error.code, "external-oracle/unknown-semantic-id");
    }
}
