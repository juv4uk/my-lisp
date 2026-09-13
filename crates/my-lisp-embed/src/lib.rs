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
    os::raw::{c_char, c_void},
    panic::{catch_unwind, AssertUnwindSafe},
    ptr,
};

/// C ABI contract version.  Hosts must compare this value before using the
/// session exports, rather than treating matching symbol names as proof of
/// compatibility.
pub const MY_LISP_EMBED_ABI_VERSION: u32 = 3;

/// Result tags that a nullary host mechanism may return through the C ABI.
pub const MY_LISP_EMBED_NIL: u32 = 0;
pub const MY_LISP_EMBED_TRUE: u32 = 1;

/// One atomic host mechanism. It must only observe or perform the requested
/// action; the caller's Lisp program keeps all policy and orchestration.
pub type MyLispEmbedNullaryFn = unsafe extern "C" fn(*mut c_void, *mut u32) -> i32;

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

fn invoke_nullary(
    callback: MyLispEmbedNullaryFn,
    context: *mut c_void,
    surface: &str,
    arguments: &[my_lisp::Value],
    span: my_lisp::Span,
) -> Result<my_lisp::Value, my_lisp::LanguageError> {
    if !arguments.is_empty() {
        return Err(my_lisp::LanguageError::new(
            my_lisp::ErrorKind::InvalidForm,
            format!("{surface} expects exactly 0 arguments"),
            span,
        ));
    }
    let mut raw_result = MY_LISP_EMBED_NIL;
    let status = unsafe { callback(context, &mut raw_result) };
    if status != 0 {
        return Err(my_lisp::LanguageError::new(
            my_lisp::ErrorKind::InvalidForm,
            format!("{surface}: host mechanism failed with status {status}"),
            span,
        ));
    }
    match raw_result {
        MY_LISP_EMBED_NIL => Ok(my_lisp::Value::Nil),
        MY_LISP_EMBED_TRUE => Ok(my_lisp::Value::truth(true)),
        _ => Err(my_lisp::LanguageError::new(
            my_lisp::ErrorKind::InvalidForm,
            format!("{surface}: host returned an unknown result tag {raw_result}"),
            span,
        )),
    }
}

/// Returns the version of the C embedding contract implemented by this DLL.
#[no_mangle]
pub extern "C" fn my_lisp_embed_abi_version() -> u32 {
    MY_LISP_EMBED_ABI_VERSION
}

/// Binds an embedding-owned opaque handle. Lisp may pass this value to a host
/// mechanism, but source syntax cannot construct or inspect its token.
#[no_mangle]
pub unsafe extern "C" fn my_lisp_embed_bind_host_handle(
    session: *mut MyLispEmbedSession,
    surface: *const c_char,
    kind: *const c_char,
    token: u64,
) -> i32 {
    if session.is_null() || surface.is_null() || kind.is_null() {
        return -1;
    }
    let decode = |text: *const c_char| {
        CStr::from_ptr(text)
            .to_str()
            .ok()
            .filter(|text| !text.is_empty())
            .map(str::to_owned)
    };
    let (Some(surface), Some(kind)) = (decode(surface), decode(kind)) else {
        return -2;
    };
    (&mut *session)
        .session
        .environment
        .define(surface, my_lisp::Value::host_handle(kind, token));
    0
}

/// Binds a zero-argument host mechanism into one canonical session.
///
/// Returns zero on success. Negative results indicate an invalid session,
/// surface, or callback. A non-zero result from the host callback becomes a
/// Lisp error for that evaluation and leaves the session usable.
#[no_mangle]
pub unsafe extern "C" fn my_lisp_embed_register_nullary(
    session: *mut MyLispEmbedSession,
    surface: *const c_char,
    callback: Option<MyLispEmbedNullaryFn>,
    context: *mut c_void,
) -> i32 {
    if session.is_null() || surface.is_null() || callback.is_none() {
        return -1;
    }
    let surface = match CStr::from_ptr(surface).to_str() {
        Ok(surface) if !surface.is_empty() => surface.to_owned(),
        _ => return -2,
    };
    let callback = callback.expect("validated above");
    let diagnostic_surface = surface.clone();
    let function = std::rc::Rc::new(
        move |arguments: &[my_lisp::Value], _environment: &my_lisp::Environment, span| {
            invoke_nullary(callback, context, &diagnostic_surface, arguments, span)
        },
    );
    (&mut *session)
        .session
        .environment
        .define(surface, my_lisp::Value::host_function(function));
    0
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
    use std::sync::atomic::{AtomicUsize, Ordering};

    static NULLARY_CALLS: AtomicUsize = AtomicUsize::new(0);

    unsafe extern "C" fn true_fact(_context: *mut c_void, out_result: *mut u32) -> i32 {
        NULLARY_CALLS.fetch_add(1, Ordering::SeqCst);
        *out_result = MY_LISP_EMBED_TRUE;
        0
    }

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
    fn reports_the_documented_abi_version() {
        assert_eq!(my_lisp_embed_abi_version(), MY_LISP_EMBED_ABI_VERSION);
    }

    #[test]
    fn nullary_host_mechanism_returns_canonical_truth_to_lisp() {
        NULLARY_CALLS.store(0, Ordering::SeqCst);
        let session = my_lisp_embed_session_new();
        assert_eq!(
            unsafe {
                my_lisp_embed_register_nullary(
                    session,
                    CString::new("гравець-присутній?").unwrap().as_ptr(),
                    Some(true_fact),
                    ptr::null_mut(),
                )
            },
            0
        );

        assert_eq!(eval(session, "(гравець-присутній?)"), "t");
        assert_eq!(NULLARY_CALLS.load(Ordering::SeqCst), 1);
        assert!(eval(session, "(гравець-присутній? 1)").starts_with("error: "));
        assert_eq!(eval(session, "(гравець-присутній?)"), "t");
        assert_eq!(NULLARY_CALLS.load(Ordering::SeqCst), 2);

        unsafe { my_lisp_embed_session_free(session) };
    }

    #[test]
    fn host_handle_binding_is_opaque_to_lisp_source() {
        let session = my_lisp_embed_session_new();
        assert_eq!(
            unsafe {
                my_lisp_embed_bind_host_handle(
                    session,
                    CString::new("гравець").unwrap().as_ptr(),
                    CString::new("cyberpunk.IScriptable").unwrap().as_ptr(),
                    42,
                )
            },
            0
        );
        assert_eq!(
            eval(session, "гравець"),
            "#<host-handle cyberpunk.IScriptable>"
        );
        assert!(!eval(session, "гравець").contains("42"));
        unsafe { my_lisp_embed_session_free(session) };
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
