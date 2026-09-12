//! RED-only specification tests for WSM-5 Task 2.
//!
//! This module is compiled only under `cfg(test)` until the tests have been
//! observed failing for the intended reason. The two translation functions are
//! wished-for API stubs, not an implementation.

use my_lisp::{Expr, ExprKind};

#[derive(Debug, Clone, PartialEq, Eq)]
struct Unsupported {
    code: &'static str,
}

fn translate_source(_source: &str) -> Result<String, Unsupported> {
    unimplemented!("exact AST to Wolfram translation")
}

fn translate_expr(_expr: &Expr) -> Result<String, Unsupported> {
    unimplemented!("exact AST to Wolfram translation")
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
