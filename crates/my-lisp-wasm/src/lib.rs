//! WebAssembly bindings exposing the canonical my-lisp engine to the browser.
//! Persistent session with core.my preloaded on first call.

use my_lisp::Session;
use my_lisp_literate::SourceMode;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use wasm_bindgen::prelude::*;

const CORE_LIB: &str = include_str!("../../../lib/core.my");

thread_local! {
    static SESSION: RefCell<Option<Session>> = const { RefCell::new(None) };
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Evaluation {
    value: String,
    output: Vec<String>,
    ast: String,
    engine: &'static str,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Diagnostic {
    from: usize,
    to: usize,
    severity: &'static str,
    message: String,
}

/// Ensures the shared session exists and has core.my preloaded.
/// Idempotent — subsequent calls are no-ops.
fn init_if_needed() {
    SESSION.with(|slot| {
        let mut guard = slot.borrow_mut();
        if guard.is_none() {
            let mut session = Session::default();
            let _ = my_lisp::eval_program(CORE_LIB, &mut session);
            *guard = Some(session);
        }
    });
}

#[wasm_bindgen]
pub fn evaluate(source: &str, mode: JsValue) -> Result<JsValue, JsValue> {
    init_if_needed();

    let mode_str = mode.as_string().unwrap_or_default();
    let source_mode = if mode_str == "markdown" { SourceMode::Literate } else { SourceMode::PureLisp };

    SESSION.with(|slot| {
        let mut guard = slot.borrow_mut();
        let session = guard.as_mut().expect("session set by init_if_needed");
        let (result, forms) = my_lisp_literate::eval_literate(source, source_mode, session)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        let evaluation = Evaluation {
            value: result.value.to_string(),
            output: result.output,
            ast: format!("{forms:#?}"),
            engine: "my-lisp · WASM",
        };

        serde_wasm_bindgen::to_value(&evaluation).map_err(|e| JsValue::from_str(&e.to_string()))
    })
}

/// Force a fresh session (clears all definitions).
#[wasm_bindgen]
pub fn reset_session() {
    SESSION.with(|slot| {
        *slot.borrow_mut() = None;
    });
}

#[wasm_bindgen]
pub fn diagnose(source: &str, mode: JsValue) -> JsValue {
    let mode_str = mode.as_string().unwrap_or_default();
    let source_mode = if mode_str == "markdown" { SourceMode::Literate } else { SourceMode::PureLisp };

    let diagnostics = match my_lisp_literate::parse_literate(source, source_mode) {
        Ok(_) => vec![],
        Err(e) => vec![Diagnostic {
            from: e.span.start,
            to: e.span.end,
            severity: "error",
            message: e.to_string(),
        }],
    };

    serde_wasm_bindgen::to_value(&diagnostics).unwrap_or(JsValue::NULL)
}

// ── native tests ──

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn core_my_definitions_available_after_init() {
        init_if_needed();
        SESSION.with(|slot| {
            let mut guard = slot.borrow_mut();
            let session = guard.as_mut().unwrap();
            let (result, _) = my_lisp_literate::eval_literate(
                "(length (quote (a b c)))",
                SourceMode::PureLisp, session)
                .expect("length should work after core.my preload");
            assert_eq!(result.value.to_string(), "3");
        });
    }

    #[test]
    fn persistent_session_preserves_definitions_across_calls() {
        reset_session();
        init_if_needed();

        // Define foo in one call
        SESSION.with(|slot| {
            let mut guard = slot.borrow_mut();
            let session = guard.as_mut().unwrap();
            let _ = my_lisp_literate::eval_literate(
                "(def foo (lambda (x) (+ x 1)))",
                SourceMode::PureLisp, session)
                .expect("def should succeed");
        });

        // Call foo in a separate eval — same session
        SESSION.with(|slot| {
            let mut guard = slot.borrow_mut();
            let session = guard.as_mut().unwrap();
            let (result, _) = my_lisp_literate::eval_literate(
                "(foo 5)",
                SourceMode::PureLisp, session)
                .expect("foo should be visible from previous eval");
            assert_eq!(result.value.to_string(), "6");
        });
    }
}
