//! The one place that embeds Guard's Rust+Lisp foundation.
//! Єдине місце, де вбудовано раст+лісп-основу Guard.
//!
//! Before this crate existed, `lib/core.my` and `lib/guard.wsm` were each
//! embedded via a separate `include_str!` in three different places
//! (`wsm-guard-slice`, `wsm-guard-facts`, `my-lisp-cli`'s `--oracle-help`),
//! and the session-setup + decision-validation sequence around them was
//! duplicated near-verbatim between the two adapter binaries. Guard is one
//! mechanism, not three: this crate is the single embed and the single
//! evaluate-and-validate path every Rust consumer shares.
//!
//! What stays compiled in (`CORE`, `GUARD`) is executable Lisp code shipped
//! with the binary, exactly like a standard library — changing it is a
//! logic change that deserves a rebuild and review. What must NOT be
//! compiled in is *data* — policy files and the `knowledge/guard-reference.wsm`
//! reference directory are read fresh from disk by their own callers
//! (`--policy PATH`, or a plain `fs::read_to_string` for the reference
//! directory), so adding a new reference entry or changing a policy never
//! requires touching Rust at all.

use my_lisp::{Session, eval_program};

/// The my-lisp language core. Single source of truth for every Guard
/// consumer that needs a session to evaluate `guard.wsm` at all.
pub const CORE: &str = include_str!("../../../lib/core.my");

/// The shared guard-finding function library (`guard-evaluate`,
/// `guard-fact-evaluate`, `make-guard-finding`, ...). Single source of
/// truth — no other crate embeds this file.
pub const GUARD: &str = include_str!("../../../lib/guard.wsm");

/// Load `CORE` and `GUARD` into a fresh session, on top of which a caller
/// can evaluate its own WSM policy and query expressions. Exposed
/// separately from `evaluate` for callers (like `--oracle-help`) whose
/// interaction shape is "load the guard library, then query it" rather
/// than "evaluate one bounded decision and validate its shape."
pub fn load_session() -> Result<Session, String> {
    let mut session = Session::default();
    eval_program(CORE, &mut session).map_err(|error| format!("core: {error}"))?;
    eval_program(GUARD, &mut session).map_err(|error| format!("guard: {error}"))?;
    Ok(session)
}

/// Load `CORE` + `GUARD` + the caller's `policy`, evaluate `call` against
/// that session, and validate that the result is one of the four decision
/// shapes Guard promises. This is the exact sequence `wsm-guard-slice` and
/// `wsm-guard-facts` each re-implemented independently before this crate
/// existed; both now delegate here.
pub fn evaluate(policy: &str, call: &str) -> Result<String, String> {
    let mut session = load_session()?;
    eval_program(policy, &mut session).map_err(|error| format!("policy: {error}"))?;
    let result = eval_program(call, &mut session).map_err(|error| format!("evaluate: {error}"))?;
    let rendered = result.value.to_string();
    if ![
        "(decision allow)",
        "(decision warn)",
        "(decision reject)",
        "(decision unknown)",
    ]
    .iter()
    .any(|decision| rendered.contains(decision))
    {
        return Err("policy-result-without-valid-decision".into());
    }
    Ok(rendered)
}

#[cfg(test)]
mod tests {
    use super::*;

    const ALLOW_POLICY: &str = r#"
      (def guard-evaluate
        (lambda (kind subject evidence)
          (make-guard-finding (quote allow) (quote confirmed) subject
            kind (quote test-contract) (quote ()) (quote no-impact)
            (quote no-action) (list evidence))))"#;

    #[test]
    fn evaluates_a_real_policy_against_the_shared_core_and_guard_library() {
        let result = evaluate(
            ALLOW_POLICY,
            "(guard-evaluate (quote read) (quote docs) (quote confirmed))",
        )
        .unwrap();
        assert!(result.contains("(decision allow)"));
        assert!(result.contains("(schema guard/1)"));
    }

    #[test]
    fn rejects_a_policy_result_without_a_valid_decision_shape() {
        let bad_policy = r#"
          (def guard-evaluate
            (lambda (kind subject evidence) (quote not-a-decision)))"#;
        let error = evaluate(
            bad_policy,
            "(guard-evaluate (quote read) (quote docs) (quote confirmed))",
        )
        .unwrap_err();
        assert_eq!(error, "policy-result-without-valid-decision");
    }

    #[test]
    fn policy_reload_changes_decision_without_recompiling_rust() {
        let warn_policy = ALLOW_POLICY.replace("(quote allow)", "(quote warn)");
        let allow = evaluate(
            ALLOW_POLICY,
            "(guard-evaluate (quote read) (quote docs) (quote confirmed))",
        )
        .unwrap();
        let warn = evaluate(
            &warn_policy,
            "(guard-evaluate (quote read) (quote docs) (quote confirmed))",
        )
        .unwrap();
        assert!(allow.contains("(decision allow)"));
        assert!(warn.contains("(decision warn)"));
    }

    #[test]
    fn load_session_alone_leaves_guard_wsm_functions_ready_to_call() {
        let mut session = load_session().unwrap();
        let result = eval_program("(guard-decision? (quote allow))", &mut session);
        assert!(result.is_ok(), "{result:?}");
    }
}
