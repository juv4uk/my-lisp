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
    let is_literate = mode_str == "markdown";
    serde_wasm_bindgen::to_value(&diagnose_impl(source, is_literate)).unwrap_or(JsValue::NULL)
}

/// Non-wasm_bindgen core of diagnose() -- pulled out so native #[test]s
/// can exercise it directly (JsValue isn't constructible from a plain
/// native test without the wasm-bindgen-test harness).
fn diagnose_impl(source: &str, is_literate: bool) -> Vec<Diagnostic> {
    // extract_code/remap_offset are the same functions parse_literate()
    // itself uses internally (my-lisp-literate/src/lib.rs) -- called
    // directly here, instead of going through parse_literate(), so the
    // offset_maps are available afterward to remap arity_diagnostics'
    // spans too, not just a parse error's.
    let (concatenated, offset_maps) = my_lisp_literate::extract_code(source, is_literate);

    let mut diagnostics: Vec<Diagnostic> = Vec::new();

    if is_literate && offset_maps.is_empty() {
        return diagnostics;
    }

    match my_lisp::parse(&concatenated) {
        Err(e) => diagnostics.push(Diagnostic {
            from: my_lisp_literate::remap_offset(e.span.start, &offset_maps),
            to: my_lisp_literate::remap_offset(e.span.end, &offset_maps),
            severity: "error",
            message: e.to_string(),
        }),
        // Only meaningful to check arity once the source actually
        // parses -- same order the native LSP server uses
        // (my-lisp-lsp/src/server.rs's publish()), which this reuses
        // rather than reimplementing: canonical language-item arities
        // from my_lisp::language_items(), not a WASM-side guess.
        Ok(_) => {
            if let Ok(arity_diags) = my_lisp_lsp::analysis::arity_diagnostics(&concatenated) {
                for d in arity_diags {
                    diagnostics.push(Diagnostic {
                        from: my_lisp_literate::remap_offset(d.span.start, &offset_maps),
                        to: my_lisp_literate::remap_offset(d.span.end, &offset_maps),
                        severity: "error",
                        message: d.message,
                    });
                }
            }
        }
    }

    diagnostics
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

    #[test]
    fn diagnose_reports_arity_mismatch_in_pure_lisp() {
        // Same example my-lisp-lsp/tests/e2e.rs's
        // t15_arity_diagnostics_are_conservative_and_shadow_aware uses
        // for the native LSP -- confirms the WASM side now reports the
        // same canonical diagnostic, not just parse errors.
        let diagnostics = diagnose_impl("(car 1 2)", false);
        assert_eq!(diagnostics.len(), 1);
        assert!(
            diagnostics[0].message.contains("arity: car expects 1, received 2"),
            "expected canonical arity message, got: {}",
            diagnostics[0].message
        );
        assert_eq!(diagnostics[0].severity, "error");
    }

    #[test]
    fn diagnose_reports_no_arity_diagnostic_for_valid_call() {
        let diagnostics = diagnose_impl("(car (quote (1 2)))", false);
        assert!(diagnostics.is_empty(), "valid call must not be flagged: {diagnostics:?}");
    }

    #[test]
    fn diagnose_remaps_arity_span_correctly_in_literate_mode() {
        // The bad call sits inside a fenced ```my-lisp block, offset
        // from the start of the document by the markdown prose before
        // it -- this is exactly the case extract_code's offset_maps
        // exist for. If remapping were wrong (e.g. reporting the
        // position within the *extracted* code instead of the original
        // document), `from`/`to` would point at the wrong characters.
        let source = "# Doc\n\nSome prose.\n\n```my-lisp\n(car 1 2)\n```\n";
        let diagnostics = diagnose_impl(source, true);
        assert_eq!(diagnostics.len(), 1);
        let d = &diagnostics[0];
        assert_eq!(
            &source[d.from..d.to],
            "(car 1 2)",
            "remapped span must point at the actual call in the original document, got: {:?}",
            &source[d.from..d.to]
        );
    }
}
