//! Immutable semantic registry for Canon 0 + McCarthy7.
//!
//! This module is deliberately *not* an `Environment`.  Environments bind
//! names and remain shadowable; this table binds canonical identities to
//! semantics and has no mutation API.  Surface names may be rebound without
//! changing the canonical operation they originally denote.

use super::special_forms::{car_value, cdr_value, cons_values, eq_values};
use crate::{Environment, ErrorKind, LanguageError, Span, Value};
use std::rc::Rc;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
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
    pub historical: &'static str,
    pub ukrainian: &'static str,
    pub sanskrit: &'static str,
}

/// The immutable 0 + 7 registry.  There is intentionally no setter, mutable
/// static, `Environment`, or user-visible `define` path here.
pub(crate) const CANON: [CanonEntry; 8] = [
    CanonEntry {
        identity: CanonicalIdentity::EmptyList,
        kind: CanonicalKind::GroundValue,
        historical: "()",
        ukrainian: "()",
        sanskrit: "()",
    },
    CanonEntry {
        identity: CanonicalIdentity::Quote,
        kind: CanonicalKind::SpecialForm,
        historical: "quote",
        ukrainian: "як-є",
        sanskrit: "svarūpa",
    },
    CanonEntry {
        identity: CanonicalIdentity::Atom,
        kind: CanonicalKind::ValuePrimitive,
        historical: "atom",
        ukrainian: "атом?",
        sanskrit: "aṇu",
    },
    CanonEntry {
        identity: CanonicalIdentity::Eq,
        kind: CanonicalKind::ValuePrimitive,
        historical: "eq",
        ukrainian: "тотожне?",
        sanskrit: "abheda",
    },
    CanonEntry {
        identity: CanonicalIdentity::Cons,
        kind: CanonicalKind::ValuePrimitive,
        historical: "cons",
        ukrainian: "сполучити",
        sanskrit: "saṃyuj",
    },
    CanonEntry {
        identity: CanonicalIdentity::Car,
        kind: CanonicalKind::ValuePrimitive,
        historical: "car",
        ukrainian: "перше",
        sanskrit: "ādi",
    },
    CanonEntry {
        identity: CanonicalIdentity::Cdr,
        kind: CanonicalKind::ValuePrimitive,
        historical: "cdr",
        ukrainian: "решта",
        sanskrit: "śeṣa",
    },
    CanonEntry {
        identity: CanonicalIdentity::Cond,
        kind: CanonicalKind::SpecialForm,
        historical: "cond",
        ukrainian: "за-умовою",
        sanskrit: "anukrama",
    },
];

pub(crate) fn identity_for_surface(name: &str) -> Option<CanonicalIdentity> {
    CANON.iter()
        .find(|entry| {
            entry.historical == name || entry.ukrainian == name || entry.sanskrit == name
        })
        .map(|entry| entry.identity)
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

/// Materialize the canonical first-class value for an identity.  Each call
/// creates a callable handle backed by the same hard-coded semantic path; no
/// environment lookup participates in choosing the implementation.
pub(crate) fn value(identity: CanonicalIdentity) -> Option<Value> {
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

/// Resolve a *surface spelling* to a canonical first-class value.  The caller
/// should consult the ordinary environment first so lexical shadowing remains
/// intact; this function is the immutable fallback, never another name lookup.
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
    }

    #[test]
    fn three_surfaces_resolve_to_one_identity() {
        assert_eq!(identity_for_surface("car"), Some(CanonicalIdentity::Car));
        assert_eq!(identity_for_surface("перше"), Some(CanonicalIdentity::Car));
        assert_eq!(identity_for_surface("ādi"), Some(CanonicalIdentity::Car));
    }

    #[test]
    fn empty_list_is_a_value_not_a_primitive_operation() {
        assert_eq!(ground_value(CanonicalIdentity::EmptyList), Some(Value::Nil));
        assert!(value(CanonicalIdentity::Quote).is_none());
        assert!(value(CanonicalIdentity::Cond).is_none());
    }
}
