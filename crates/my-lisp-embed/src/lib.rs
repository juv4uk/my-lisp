//! Stateful C ABI embedding boundary for the canonical my-lisp evaluator.
//!
//! This crate is deliberately a transport boundary only.  Parsing, bootstrap,
//! evaluation, output, error rendering, and session state all come from
//! `my-lisp`; no second evaluator lives here.

use my_lisp::{
    eval_parsed_expressions_incremental, load_core_library, parse, Environment, Session,
};
use std::{
    ffi::{CStr, CString},
    os::raw::c_char,
    panic::{catch_unwind, AssertUnwindSafe},
    ptr,
};

/// Opaque owner of one canonical, persistent my-lisp session.
pub struct MyLispEmbedSession {
    session: Session,
}

fn new_session() -> Result<MyLispEmbedSession, String> {
    let mut session = Session {
        environment: Environment::root(),
    };
    load_core_library(&mut session).map_err(|error| error.render(my_lisp::CORE_LIBRARY_SOURCE))?;
    Ok(MyLispEmbedSession { session })
}

fn response_for(session: &mut MyLispEmbedSession, source: &str) -> String {
    match parse(source) {
        Ok(expressions) => {
            match eval_parsed_expressions_incremental(&expressions, &mut session.session) {
                Ok(result) => {
                    let mut response = result.output.join("\n");
                    if !response.is_empty() {
                        response.push('\n');
                    }
                    response.push_str(&result.value.to_string());
                    response
                }
                Err(error) => format!("error: {}", error.render(source)),
            }
        }
        Err(error) => format!("error: {}", error.render(source)),
    }
}

fn into_c_string(response: String) -> *mut c_char {
    CString::new(response)
        .unwrap_or_else(|_| {
            CString::new("error: result contains a NUL byte").expect("static C string")
        })
        .into_raw()
}

/// Creates a persistent canonical my-lisp session with the ordinary core
/// library loaded.  Returns null only if bootstrap unexpectedly panics.
#[no_mangle]
pub extern "C" fn my_lisp_embed_session_new() -> *mut MyLispEmbedSession {
    catch_unwind(AssertUnwindSafe(new_session))
        .ok()
        .and_then(Result::ok)
        .map(Box::new)
        .map(Box::into_raw)
        .unwrap_or(ptr::null_mut())
}

/// Evaluates UTF-8 source against this session and returns caller-owned UTF-8
/// text.  The returned pointer must be freed with `my_lisp_embed_free_string`.
/// Null source or a null session return null.
#[no_mangle]
pub unsafe extern "C" fn my_lisp_embed_eval(
    session: *mut MyLispEmbedSession,
    source: *const c_char,
) -> *mut c_char {
    if session.is_null() || source.is_null() {
        return ptr::null_mut();
    }

    let source = match CStr::from_ptr(source).to_str() {
        Ok(source) => source,
        Err(_) => return into_c_string("error: source is not valid UTF-8".to_owned()),
    };

    catch_unwind(AssertUnwindSafe(|| response_for(&mut *session, source)))
        .map(into_c_string)
        .unwrap_or_else(|_| into_c_string("error: canonical session panicked".to_owned()))
}

/// Frees a string returned by `my_lisp_embed_eval`.  A null pointer is a no-op.
#[no_mangle]
pub unsafe extern "C" fn my_lisp_embed_free_string(value: *mut c_char) {
    if !value.is_null() {
        drop(CString::from_raw(value));
    }
}

/// Frees a session returned by `my_lisp_embed_session_new`.  A null pointer is
/// a no-op.  All calls for a session must remain on its owning host thread.
#[no_mangle]
pub unsafe extern "C" fn my_lisp_embed_session_free(session: *mut MyLispEmbedSession) {
    if !session.is_null() {
        drop(Box::from_raw(session));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn eval(session: *mut MyLispEmbedSession, source: &str) -> String {
        let source = CString::new(source).unwrap();
        let raw_result = unsafe { my_lisp_embed_eval(session, source.as_ptr()) };
        assert!(!raw_result.is_null());
        let result = unsafe { CStr::from_ptr(raw_result) }
            .to_str()
            .unwrap()
            .to_owned();
        unsafe { my_lisp_embed_free_string(raw_result) };
        result
    }

    #[test]
    fn ukrainian_definition_persists_in_one_canonical_session() {
        let session = my_lisp_embed_session_new();
        assert!(!session.is_null());

        assert_eq!(eval(session, "(визначити repl-перевірка 42)"), "42");
        assert_eq!(eval(session, "repl-перевірка"), "42");

        unsafe { my_lisp_embed_session_free(session) };
    }

    #[test]
    fn ukrainian_closure_persists_in_one_canonical_session() {
        let session = my_lisp_embed_session_new();
        assert!(!session.is_null());

        let definition = eval(
            session,
            "(визначити подвоїти (функція (значення) (+ значення значення)))",
        );
        assert_eq!(definition, "<lambda>");
        assert_eq!(eval(session, "(подвоїти 21)"), "42");

        unsafe { my_lisp_embed_session_free(session) };
    }

    #[test]
    fn language_errors_return_text_without_destroying_the_session() {
        let session = my_lisp_embed_session_new();
        assert!(!session.is_null());

        assert!(eval(session, "(car 7)").starts_with("error: "));
        assert_eq!(eval(session, "(+ 20 22)"), "42");

        unsafe { my_lisp_embed_session_free(session) };
    }
}
