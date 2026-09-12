//! Immutable evaluator meaning for Canon 0 + McCarthy7.
//!
//! Canon is deliberately *not* an `Environment`. Stable human/symbolic
//! spellings live in `lib/surface/semantic-registry.wsm` and are projected to
//! opaque numeric IDs by the shared registry module. This module owns only the
//! finite mapping from those IDs to canonical evaluator meaning, plus Canon 0.

use super::special_forms::{car_value, cdr_value, cons_values, eq_values};
use crate::{semantic_registry, Environment, ErrorKind, LanguageError, Span, Value};
use std::rc::Rc;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) enum CanonicalIdentity {
    EmptyList,
    Quote,
    Atom,
    Eq,
    Cons,
    Car,
    Cdr,
    Cond,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CanonicalKind {
    GroundValue,
    ValuePrimitive,
    SpecialForm,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct CanonEntry {
    pub identity: CanonicalIdentity,
    pub kind: CanonicalKind,
    pub semantic_id: Option<&'static str>,
}

pub(crate) const QUOTE_SEMANTIC_ID: &str = "0001";
pub(crate) const ATOM_SEMANTIC_ID: &str = "0002";
pub(crate) const EQ_SEMANTIC_ID: &str = "0003";
pub(crate) const CONS_SEMANTIC_ID: &str = "0004";
pub(crate) const CAR_SEMANTIC_ID: &str = "0005";
pub(crate) const CDR_SEMANTIC_ID: &str = "0006";
pub(crate) const COND_SEMANTIC_ID: &str = "0007";

/// Canon 0 has no surface row: the empty list is the ground object itself.
/// McCarthy7 meanings are keyed only by opaque numeric semantic IDs here.
pub(crate) const CANON: [CanonEntry; 8] = [
    CanonEntry {
        identity: CanonicalIdentity::EmptyList,
        kind: CanonicalKind::GroundValue,
        semantic_id: None,
    },
    CanonEntry {
        identity: CanonicalIdentity::Quote,
        kind: CanonicalKind::SpecialForm,
        semantic_id: Some(QUOTE_SEMANTIC_ID),
    },
    CanonEntry {
        identity: CanonicalIdentity::Atom,
        kind: CanonicalKind::ValuePrimitive,
        semantic_id: Some(ATOM_SEMANTIC_ID),
    },
    CanonEntry {
        identity: CanonicalIdentity::Eq,
        kind: CanonicalKind::ValuePrimitive,
        semantic_id: Some(EQ_SEMANTIC_ID),
    },
    CanonEntry {
        identity: CanonicalIdentity::Cons,
        kind: CanonicalKind::ValuePrimitive,
        semantic_id: Some(CONS_SEMANTIC_ID),
    },
    CanonEntry {
        identity: CanonicalIdentity::Car,
        kind: CanonicalKind::ValuePrimitive,
        semantic_id: Some(CAR_SEMANTIC_ID),
    },
    CanonEntry {
        identity: CanonicalIdentity::Cdr,
        kind: CanonicalKind::ValuePrimitive,
        semantic_id: Some(CDR_SEMANTIC_ID),
    },
    CanonEntry {
        identity: CanonicalIdentity::Cond,
        kind: CanonicalKind::SpecialForm,
        semantic_id: Some(COND_SEMANTIC_ID),
    },
];

fn identity_for_semantic_id(semantic_id: &str) -> Option<CanonicalIdentity> {
    CANON
        .iter()
        .find(|entry| entry.semantic_id == Some(semantic_id))
        .map(|entry| entry.identity)
}

fn semantic_id_for_identity(identity: CanonicalIdentity) -> Option<&'static str> {
    CANON
        .iter()
        .find(|entry| entry.identity == identity)
        .and_then(|entry| entry.semantic_id)
}

pub(crate) fn identity_for_surface(name: &str) -> Option<CanonicalIdentity> {
    semantic_registry::semantic_id_for_surface(name).and_then(identity_for_semantic_id)
}

pub(crate) fn is_reserved_surface(name: &str) -> bool {
    identity_for_surface(name).is_some()
}

/// True for any admitted surface of `quote` specifically (semantic ID
/// `0001`) — `quote`/`як-є`/`svarūpa`/`'`, not just the English spelling.
/// Exposed narrowly via `crate::is_quote_surface_name` for tooling that
/// must distinguish "this list's head is quote" from "this list's head is
/// some other Canon identity," per the same routing every surface already
/// shares.
pub(crate) fn is_quote_identity(name: &str) -> bool {
    identity_for_surface(name) == Some(CanonicalIdentity::Quote)
}

pub(crate) fn ensure_bindable(name: &str, span: Span) -> Result<(), LanguageError> {
    let Some(identity) = identity_for_surface(name) else {
        return Ok(());
    };
    Err(LanguageError::new(
        ErrorKind::InvalidForm,
        format!(
            "canonical name is immutable · канонічне ім'я незмінне · kanonischer Name ist unveränderlich: {name} -> {identity:?}"
        ),
        span,
    ))
}

pub(crate) fn ground_value(identity: CanonicalIdentity) -> Option<Value> {
    match identity {
        CanonicalIdentity::EmptyList => Some(Value::Nil),
        _ => None,
    }
}

fn exact_args(
    identity: &'static str,
    args: &[Value],
    expected: usize,
    span: Span,
) -> Result<(), LanguageError> {
    if args.len() == expected {
        return Ok(());
    }
    Err(LanguageError::new(
        ErrorKind::Arity,
        format!(
            "{identity}: expected / ochikuvalosia / erwartet {expected}; received / otrymano / erhalten {}",
            args.len()
        ),
        span,
    ))
}

/// Resolve an opaque Canon semantic ID to its current host implementation.
/// The ID is the call-target identity; the Rust closure is only a projection.
fn invoke_semantic_callable(
    semantic_id: &'static str,
    args: &[Value],
    environment: &Environment,
    span: Span,
) -> Result<Value, LanguageError> {
    match identity_for_semantic_id(semantic_id) {
        Some(CanonicalIdentity::Atom) => {
            exact_args("PRIM_ATOM", args, 1, span)?;
            Ok(Value::truth(args[0].is_atom()))
        }
        Some(CanonicalIdentity::Eq) => {
            exact_args("PRIM_EQ", args, 2, span)?;
            eq_values(args[0].clone(), args[1].clone(), span)
        }
        Some(CanonicalIdentity::Cons) => {
            exact_args("PRIM_CONS", args, 2, span)?;
            cons_values(args[0].clone(), args[1].clone(), environment, span)
        }
        Some(CanonicalIdentity::Car) => {
            exact_args("PRIM_CAR", args, 1, span)?;
            car_value(&args[0], span)
        }
        Some(CanonicalIdentity::Cdr) => {
            exact_args("PRIM_CDR", args, 1, span)?;
            cdr_value(&args[0], span)
        }
        Some(CanonicalIdentity::EmptyList | CanonicalIdentity::Quote | CanonicalIdentity::Cond)
        | None => Err(LanguageError::new(
            ErrorKind::Type,
            format!(
                "semantic identity is not a callable Canon value · семантична тотожність не є викличним значенням Канону · semantische Identität ist kein aufrufbarer Kanon-Wert: {semantic_id}"
            ),
            span,
        )),
    }
}

/// Materialize a host implementation projection for a semantic call target.
/// A fresh Rc is intentional: allocation identity must not define language
/// identity. `eq` observes only the opaque semantic ID carried in `name`.
fn semantic_callable(semantic_id: &'static str) -> Value {
    Value::Builtin(Rc::new(crate::value::Builtin {
        name: semantic_id,
        func: Rc::new(move |args, environment, span| {
            invoke_semantic_callable(semantic_id, args, environment, span)
        }),
    }))
}

fn semantic_callable_id(value: &Value) -> Option<&'static str> {
    let Value::Builtin(builtin) = value else {
        return None;
    };
    let semantic_id = builtin.name;
    let identity = identity_for_semantic_id(semantic_id)?;
    matches!(
        identity,
        CanonicalIdentity::Atom
            | CanonicalIdentity::Eq
            | CanonicalIdentity::Cons
            | CanonicalIdentity::Car
            | CanonicalIdentity::Cdr
    )
    .then_some(semantic_id)
}

/// When at least one side is a Canon semantic callable, answer equality using
/// only the opaque semantic ID. `None` delegates ordinary non-Canon values to
/// their existing equality semantics.
pub(crate) fn same_semantic_callable_identity(left: &Value, right: &Value) -> Option<bool> {
    match (semantic_callable_id(left), semantic_callable_id(right)) {
        (Some(left), Some(right)) => Some(left == right),
        (Some(_), None) | (None, Some(_)) => Some(false),
        (None, None) => None,
    }
}

/// Return a first-class host projection for a canonical identity. Special forms
/// deliberately have no value representation; they remain syntax-only.
pub(crate) fn value(identity: CanonicalIdentity) -> Option<Value> {
    match identity {
        CanonicalIdentity::EmptyList => Some(Value::Nil),
        CanonicalIdentity::Quote | CanonicalIdentity::Cond => None,
        CanonicalIdentity::Atom
        | CanonicalIdentity::Eq
        | CanonicalIdentity::Cons
        | CanonicalIdentity::Car
        | CanonicalIdentity::Cdr => semantic_id_for_identity(identity).map(semantic_callable),
    }
}

pub(crate) fn value_for_surface(name: &str) -> Option<Value> {
    identity_for_surface(name).and_then(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canon_has_exactly_ground_plus_seven() {
        assert_eq!(CANON.len(), 8);
        assert_eq!(CANON[0].identity, CanonicalIdentity::EmptyList);
        assert_eq!(CANON[0].kind, CanonicalKind::GroundValue);
        assert_eq!(CANON[0].semantic_id, None);
    }

    #[test]
    fn canon_meanings_are_selected_only_by_numeric_semantic_identity() {
        assert_eq!(
            identity_for_semantic_id(QUOTE_SEMANTIC_ID),
            Some(CanonicalIdentity::Quote)
        );
        assert_eq!(
            identity_for_semantic_id(CAR_SEMANTIC_ID),
            Some(CanonicalIdentity::Car)
        );
        assert_eq!(identity_for_semantic_id("0104"), None);
    }

    #[test]
    fn every_admitted_surface_for_one_semantic_id_resolves_to_one_identity() {
        // Which spellings mean "car" is a registry FACT, not a Rust literal
        // to enumerate here -- read them from the registry so this test
        // keeps meaning "Canon routes every admitted surface for 0005 to
        // the same identity" even if the admitted spellings change.
        let surfaces = semantic_registry::admitted_surfaces_for_semantic_id(CAR_SEMANTIC_ID);
        assert!(
            surfaces.len() >= 2,
            "0005 (car) should admit at least two surfaces for this invariant to be meaningful, \
             got {surfaces:?}"
        );
        for surface in &surfaces {
            assert_eq!(
                identity_for_surface(surface),
                Some(CanonicalIdentity::Car),
                "registry-admitted surface {surface:?} did not route to CanonicalIdentity::Car"
            );
        }
    }

    #[test]
    fn numeric_canon_identity_uses_the_same_semantic_call_target() {
        assert_eq!(
            identity_for_surface(CAR_SEMANTIC_ID),
            Some(CanonicalIdentity::Car)
        );
        let human_surface = semantic_registry::admitted_surfaces_for_semantic_id(CAR_SEMANTIC_ID)
            .into_iter()
            .next()
            .expect("0005 (car) should admit at least one human surface");
        let numeric = value_for_surface(CAR_SEMANTIC_ID).expect("numeric Canon identity");
        let human = value_for_surface(human_surface).expect("registry-admitted Canon surface");
        let (Value::Builtin(numeric), Value::Builtin(human)) = (&numeric, &human) else {
            panic!("0005 must materialize as a first-class host projection");
        };
        assert_eq!(numeric.name, CAR_SEMANTIC_ID);
        assert_eq!(human.name, CAR_SEMANTIC_ID);
        assert!(
            !Rc::ptr_eq(numeric, human),
            "semantic identity witness must not depend on sharing one Rust allocation"
        );
        assert_eq!(same_semantic_callable_identity(&numeric.into(), &human.into()), Some(true));
    }

    #[test]
    fn every_admitted_surface_shares_identity_without_sharing_handle() {
        let surfaces = semantic_registry::admitted_surfaces_for_semantic_id(CAR_SEMANTIC_ID);
        assert!(
            surfaces.len() >= 2,
            "0005 (car) should admit at least two surfaces for this invariant to be meaningful, \
             got {surfaces:?}"
        );

        let values = surfaces
            .iter()
            .map(|surface| {
                value_for_surface(surface).unwrap_or_else(|| {
                    panic!("registry-admitted surface {surface:?} should route through Canon")
                })
            })
            .collect::<Vec<_>>();

        for (surface, value) in surfaces.iter().zip(values.iter()) {
            assert_eq!(
                semantic_callable_id(value),
                Some(CAR_SEMANTIC_ID),
                "surface {surface:?} did not carry Canon semantic identity 0005"
            );
            assert_eq!(
                same_semantic_callable_identity(&values[0], value),
                Some(true),
                "surface {surface:?} did not share semantic callable identity 0005"
            );
        }

        let (Value::Builtin(first), Value::Builtin(second)) = (&values[0], &values[1]) else {
            panic!("0005 surfaces must materialize as first-class host projections");
        };
        assert!(
            !Rc::ptr_eq(first, second),
            "the witness must prove semantic identity survives distinct Rust allocations"
        );
    }

    #[test]
    fn synthetic_registry_constructively_controls_canon_routing() {
        const SYNTHETIC: &str = "(0001 (xx comet stable))\n(0005 (xx asteroid stable))";
        let index = semantic_registry::build_surface_index(SYNTHETIC);
        let route = |surface: &str| {
            index
                .get(surface)
                .copied()
                .and_then(identity_for_semantic_id)
        };

        assert_eq!(route("comet"), Some(CanonicalIdentity::Quote));
        assert_eq!(route("asteroid"), Some(CanonicalIdentity::Car));
        assert_eq!(route("quote"), None);
        assert_eq!(route("car"), None);
    }

    #[test]
    fn registry_rows_without_canon_meaning_do_not_become_canon() {
        assert_eq!(semantic_registry::semantic_id_for_surface("+"), Some("0104"));
        assert_eq!(identity_for_surface("+"), None);
    }

    #[test]
    fn canonical_surface_names_are_reserved() {
        for name in [
            "quote", "як-є", "svarūpa", "atom", "атом?", "aṇu", "eq", "тотожне?",
            "abheda", "cons", "сполучити", "saṃyuj", "car", "перше", "ādi", "cdr",
            "решта", "śeṣa", "cond", "за-умовою", "anukrama",
        ] {
            assert!(is_reserved_surface(name), "Canon spelling must be reserved: {name}");
        }
        assert!(!is_reserved_surface("map"));
        assert!(!is_reserved_surface("відобразити"));
    }

    #[test]
    fn empty_list_is_a_value_not_a_primitive_operation() {
        assert_eq!(ground_value(CanonicalIdentity::EmptyList), Some(Value::Nil));
        assert!(value(CanonicalIdentity::Quote).is_none());
        assert!(value(CanonicalIdentity::Cond).is_none());
    }
}
