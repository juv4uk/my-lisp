//! Immutable evaluator meaning for Canon 0 + McCarthy7.
//!
//! Canon is deliberately *not* an `Environment`. Stable human/symbolic
//! spellings live in `lib/surface/semantic-registry.wsm` and are projected to
//! opaque numeric IDs by the shared registry module. This module owns only the
//! finite mapping from those IDs to canonical evaluator meaning, plus Canon 0.

use super::special_forms::{car_value, cdr_value, cons_values, eq_values};
use crate::{semantic_registry, Environment, ErrorKind, LanguageError, Span, Value};
use std::{collections::HashMap, rc::Rc};

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

fn builtin(
    identity: &'static str,
    func: impl Fn(&[Value], &Environment, Span) -> Result<Value, LanguageError> + 'static,
) -> Value {
    Value::Builtin(Rc::new(crate::value::Builtin {
        name: identity,
        func: Rc::new(func),
    }))
}

fn materialize_value(identity: CanonicalIdentity) -> Option<Value> {
    match identity {
        CanonicalIdentity::EmptyList => ground_value(identity),
        CanonicalIdentity::Atom => Some(builtin("PRIM_ATOM", |args, _env, span| {
            exact_args("PRIM_ATOM", args, 1, span)?;
            Ok(Value::truth(args[0].is_atom()))
        })),
        CanonicalIdentity::Eq => Some(builtin("PRIM_EQ", |args, _env, span| {
            exact_args("PRIM_EQ", args, 2, span)?;
            eq_values(args[0].clone(), args[1].clone(), span)
        })),
        CanonicalIdentity::Cons => Some(builtin("PRIM_CONS", |args, env, span| {
            exact_args("PRIM_CONS", args, 2, span)?;
            cons_values(args[0].clone(), args[1].clone(), env, span)
        })),
        CanonicalIdentity::Car => Some(builtin("PRIM_CAR", |args, _env, span| {
            exact_args("PRIM_CAR", args, 1, span)?;
            car_value(&args[0], span)
        })),
        CanonicalIdentity::Cdr => Some(builtin("PRIM_CDR", |args, _env, span| {
            exact_args("PRIM_CDR", args, 1, span)?;
            cdr_value(&args[0], span)
        })),
        CanonicalIdentity::Quote | CanonicalIdentity::Cond => None,
    }
}

fn build_value_registry() -> HashMap<CanonicalIdentity, Value> {
    [
        CanonicalIdentity::Atom,
        CanonicalIdentity::Eq,
        CanonicalIdentity::Cons,
        CanonicalIdentity::Car,
        CanonicalIdentity::Cdr,
    ]
    .into_iter()
    .map(|identity| {
        (
            identity,
            materialize_value(identity).expect("callable Canon identity must materialize"),
        )
    })
    .collect()
}

thread_local! {
    /// One immutable callable handle per Canon identity per evaluator thread.
    /// Every stable registry spelling resolves to clones of these same `Rc` handles.
    static CANON_VALUES: HashMap<CanonicalIdentity, Value> = build_value_registry();
}

/// Return the stable first-class value for a canonical identity. Special forms
/// deliberately have no value representation; they remain syntax-only.
pub(crate) fn value(identity: CanonicalIdentity) -> Option<Value> {
    if identity == CanonicalIdentity::EmptyList {
        return Some(Value::Nil);
    }
    CANON_VALUES.with(|values| values.get(&identity).cloned())
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
    fn numeric_canon_identity_uses_the_same_evaluator_meaning() {
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
            panic!("PRIM_CAR must be a first-class builtin value");
        };
        assert!(Rc::ptr_eq(numeric, human));
    }

    #[test]
    fn every_admitted_surface_for_one_semantic_id_shares_one_stable_handle() {
        // Which spellings mean "car" (en/uk/sa/...) is a semantic-registry
        // FACT, owned by the registry data, not Rust knowledge -- this test
        // asserts only the Rust-implementation INVARIANT: whatever surfaces
        // the registry admits for one semantic identity, Canon materializes
        // exactly one shared callable handle for all of them. Read the real
        // admitted surfaces from the registry itself instead of hardcoding
        // "car"/"перше"/"ādi" as literals, so this test still passes
        // unchanged if the registry's admitted spellings for 0005 ever
        // change, and still fails if Canon ever gives two of them distinct
        // handles.
        let surfaces = semantic_registry::admitted_surfaces_for_semantic_id(CAR_SEMANTIC_ID);
        assert!(
            surfaces.len() >= 2,
            "0005 (car) should admit at least two surfaces for this invariant to be meaningful, \
             got {surfaces:?}"
        );

        let handles: Vec<Rc<crate::value::Builtin>> = surfaces
            .iter()
            .map(|surface| {
                let value = value_for_surface(surface)
                    .unwrap_or_else(|| panic!("registry-admitted surface {surface:?} should route through Canon"));
                let Value::Builtin(ref handle) = value else {
                    panic!("PRIM_CAR must be a first-class builtin value for surface {surface:?}");
                };
                handle.clone()
            })
            .collect();

        let first = &handles[0];
        for (surface, handle) in surfaces.iter().zip(handles.iter()) {
            assert!(
                Rc::ptr_eq(first, handle),
                "surface {surface:?} did not share Canon's one stable handle for 0005"
            );
        }
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
