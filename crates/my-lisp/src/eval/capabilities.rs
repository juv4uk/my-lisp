//! capabilities.rs - the registration point for host capabilities.
//!
//! The canonical core ships ZERO host capabilities: no filesystem, no
//! processes, no sockets. What it ships instead is this registry: a host
//! adapter (the `my-lisp-host` crate, a WASM shim, an embedder) may
//! install named special forms at startup. Until something registers a
//! name, evaluating it falls through to ordinary function application
//! and fails `UnknownSymbol` like any other unbound name - the same
//! fail-named discipline as everything else (S2).
//!
//! This makes "capability-free core" physically true rather than a
//! policy statement: the OS-touching code lives outside this crate, and
//! a build that never calls the installer cannot reach it.

use super::EvalStep;
use crate::{Environment, Expr, LanguageError, Span, Value};
use std::collections::BTreeMap;
use std::sync::OnceLock;

/// Signature of one installed capability handler. A plain function
/// pointer keeps the registry `Copy`/`Send` and forbids stateful
/// closures - capabilities get their state (allowlists, etc.) through
/// the `Environment`, exactly like every kernel primitive.
pub type HostFn = fn(&[Expr], &Environment, Span) -> Result<Value, LanguageError>;

fn registry() -> &'static std::sync::RwLock<BTreeMap<String, HostFn>> {
    static REGISTRY: OnceLock<std::sync::RwLock<BTreeMap<String, HostFn>>> = OnceLock::new();
    REGISTRY.get_or_init(|| std::sync::RwLock::new(BTreeMap::new()))
}

/// Install one capability under its surface-form name (e.g. "read-file").
/// Re-registering the same name replaces the previous handler, so an
/// embedder can override or withdraw capabilities deliberately.
pub fn register_capability(name: &str, handler: HostFn) {
    registry()
        .write()
        .expect("capability registry poisoned")
        .insert(name.to_string(), handler);
}

/// Remove one previously installed capability.
pub fn unregister_capability(name: &str) {
    if let Ok(mut map) = registry().write() {
        map.remove(name);
    }
}

/// True when a capability with this name is currently installed.
pub fn capability_installed(name: &str) -> bool {
    registry()
        .read()
        .map(|map| map.contains_key(name))
        .unwrap_or(false)
}

/// Names of all installed capabilities, sorted (for diagnostics/UIs).
pub fn installed_capabilities() -> Vec<String> {
    registry()
        .read()
        .map(|map| map.keys().cloned().collect())
        .unwrap_or_default()
}

/// Dispatch fallback in the evaluator: consulted after the kernel's own
/// special forms and primitives, before ordinary function application.
pub(crate) fn dispatch_capability(
    name: &str,
    arguments: &[Expr],
    environment: &Environment,
    span: Span,
) -> Option<Result<EvalStep, LanguageError>> {
    let handler = {
        let map = registry().read().ok()?;
        map.get(name).copied()?
    };
    Some(handler(arguments, environment, span).map(EvalStep::Value))
}
