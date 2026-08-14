//! File-system primitives: `read-file`/`write-file` and their byte-level
//! counterparts (`read-file-bytes`/`write-file-bytes`), plus the raw
//! `read_file`/`write_file`/`read_file_bytes`/`write_file_bytes` host calls
//! `evaluate_load` (in `io`) also needs — kept `pub(super)` so both
//! submodules can share the one path to the filesystem rather than each
//! having its own.

use super::core::exact_arity;
use crate::eval::evaluate;
use crate::{Environment, ErrorKind, Exactness, Expr, LanguageError, Span, Value};
use std::rc::Rc;

/// The write-side counterpart to `read-file` (PLAN.md item 13) — one
/// primitive that opens and writes in a single step, the same shape
/// `read-file` already uses for opening and reading, rather than a
/// separate stateful file-handle value: the language has no mutable
/// cells or handles to represent one, and none of `read-file`/`load`
/// needed one either. Always creates or truncates-and-overwrites the
/// target file (`std::fs::write`'s own semantics), never appends —
/// append is a separate, not-yet-decided capability, not silently
/// folded into this one.
pub(crate) fn evaluate_read_file(
    arguments: &[Expr],
    environment: &Environment,
    span: Span,
) -> Result<Value, LanguageError> {
    exact_arity("read-file", arguments, 1, span)?;
    let evaluated = evaluate(&arguments[0], environment)?;
    let Value::String(ref path) = evaluated else {
        return Err(LanguageError::new(
            ErrorKind::Type,
            "read-file expects a string path · read-file ochikuie riadok-shliakh · read-file erwartet einen String-Pfad",
            span,
        ));
    };
    let contents = read_file(path, span)?;
    Ok(Value::String(Rc::from(contents.as_str())))
}

/// `(read-dir path)` — the directory-listing counterpart to `read-file`,
/// needed to load a whole registry directory (e.g. the dhātu YAML files
/// under `panini/registry/dhatu/`) from inside My Lisp itself rather than
/// hard-coding a file list. Returns the directory's entry names as a list
/// of strings, in filesystem order, without filtering; the caller decides
/// which names to keep (e.g. the `*.yaml` suffix). Reads the directory
/// only; it does not recurse and does not stat entries.
pub(crate) fn evaluate_read_dir(
    arguments: &[Expr],
    environment: &Environment,
    span: Span,
) -> Result<Value, LanguageError> {
    exact_arity("read-dir", arguments, 1, span)?;
    let evaluated = evaluate(&arguments[0], environment)?;
    let Value::String(ref path) = evaluated else {
        return Err(LanguageError::new(
            ErrorKind::Type,
            "read-dir expects a string path · read-dir ochikuie riadok-shliakh · read-dir erwartet einen String-Pfad",
            span,
        ));
    };
    let entries = read_dir(path, span)?;
    Ok(Value::list(
        entries
            .into_iter()
            .map(|name| Value::String(Rc::from(name))),
    ))
}

pub(crate) fn evaluate_write_file(
    arguments: &[Expr],
    environment: &Environment,
    span: Span,
) -> Result<Value, LanguageError> {
    exact_arity("write-file", arguments, 2, span)?;
    let path_value = evaluate(&arguments[0], environment)?;
    let Value::String(ref path) = path_value else {
        return Err(LanguageError::new(
            ErrorKind::Type,
            "write-file expects a string path · write-file ochikuie riadok-shliakh · write-file erwartet einen String-Pfad",
            span,
        ));
    };
    let content_value = evaluate(&arguments[1], environment)?;
    let Value::String(ref content) = content_value else {
        return Err(LanguageError::new(
            ErrorKind::Type,
            "write-file expects a string as its second argument · write-file ochikuie riadok druhym arhumentom · write-file erwartet eine Zeichenkette als zweites Argument",
            span,
        ));
    };
    write_file(path, content, span)?;
    Ok(content_value)
}

/// `(write-file-bytes path byte-list)` (PLAN.md item 22) — the byte-level
/// counterpart to `write-file`: `byte-list` is a list of fixnums 0-255,
/// written as raw bytes (`std::fs::write(path, &bytes)` over a `Vec<u8>`),
/// never through `&str`. `write-file` can only ever produce valid UTF-8 —
/// no primitive in the language can build a string containing an
/// arbitrary byte (no char-code/integer->char, no bytevector type), so
/// writing a real binary (compiled machine code, any non-UTF-8 format)
/// was impossible before this.
pub(crate) fn evaluate_write_file_bytes(
    arguments: &[Expr],
    environment: &Environment,
    span: Span,
) -> Result<Value, LanguageError> {
    exact_arity("write-file-bytes", arguments, 2, span)?;
    let path_value = evaluate(&arguments[0], environment)?;
    let Value::String(ref path) = path_value else {
        return Err(LanguageError::new(
            ErrorKind::Type,
            "write-file-bytes expects a string path · write-file-bytes ochikuie riadok-shliakh · write-file-bytes erwartet einen String-Pfad",
            span,
        ));
    };
    let bytes_value = evaluate(&arguments[1], environment)?;
    let bytes = expect_byte_list(&bytes_value, arguments[1].span)?;
    write_file_bytes(path, &bytes, span)?;
    Ok(bytes_value)
}

/// `(read-file-bytes path)` (PLAN.md item 22) — the byte-level counterpart
/// to `read-file`: returns the file's raw bytes as a list of fixnums
/// 0-255, not a UTF-8-decoded string, which would fail outright — or
/// worse, silently corrupt — on a non-UTF-8 file.
pub(crate) fn evaluate_read_file_bytes(
    arguments: &[Expr],
    environment: &Environment,
    span: Span,
) -> Result<Value, LanguageError> {
    exact_arity("read-file-bytes", arguments, 1, span)?;
    let evaluated = evaluate(&arguments[0], environment)?;
    let Value::String(ref path) = evaluated else {
        return Err(LanguageError::new(
            ErrorKind::Type,
            "read-file-bytes expects a string path · read-file-bytes ochikuie riadok-shliakh · read-file-bytes erwartet einen String-Pfad",
            span,
        ));
    };
    let bytes = read_file_bytes(path, span)?;
    Ok(Value::list(
        bytes
            .into_iter()
            .map(|byte| Value::Number(byte as f64, Exactness::Exact)),
    ))
}

fn expect_byte_list(value: &Value, span: Span) -> Result<Vec<u8>, LanguageError> {
    let mut bytes = Vec::new();
    let mut current = value;
    loop {
        match current {
            Value::Nil => return Ok(bytes),
            Value::Pair(head, tail) => {
                let Value::Number(number, _) = **head else {
                    return Err(LanguageError::new(
                        ErrorKind::Type,
                        "write-file-bytes expects a list of integers 0-255 · write-file-bytes ochikuie spysok tsilykh chysel 0-255 · write-file-bytes erwartet eine Liste von Ganzzahlen 0-255",
                        span,
                    ));
                };
                if number.fract() != 0.0 || !(0.0..=255.0).contains(&number) {
                    return Err(LanguageError::new(
                        ErrorKind::Type,
                        "write-file-bytes expects each element to be an integer between 0 and 255 · write-file-bytes ochikuie, shchob kozhen element buv tsilym chyslom vid 0 do 255 · write-file-bytes erwartet, dass jedes Element eine Ganzzahl zwischen 0 und 255 ist",
                        span,
                    ));
                }
                bytes.push(number as u8);
                current = tail;
            }
            _ => {
                return Err(LanguageError::new(
                    ErrorKind::Type,
                    "write-file-bytes expects a proper list of integers 0-255 · write-file-bytes ochikuie pravylnyi spysok tsilykh chysel 0-255 · write-file-bytes erwartet eine echte Liste von Ganzzahlen 0-255",
                    span,
                ))
            }
        }
    }
}

/// Shared with `io::evaluate_load` — `pub(super)` so both submodules of
/// `special_forms` use the one path to the filesystem.
#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn read_file(path: &str, span: Span) -> Result<String, LanguageError> {
    std::fs::read_to_string(path).map_err(|error| {
        LanguageError::new(
            ErrorKind::InvalidForm,
            format!("load: failed to read file {path}: {error}"),
            span,
        )
    })
}

#[cfg(target_arch = "wasm32")]
pub(crate) fn read_file(_path: &str, span: Span) -> Result<String, LanguageError> {
    Err(LanguageError::new(
        ErrorKind::InvalidForm,
        "load: file system access is not available in this build",
        span,
    ))
}

#[cfg(not(target_arch = "wasm32"))]
fn read_dir(path: &str, span: Span) -> Result<Vec<String>, LanguageError> {
    let reader = std::fs::read_dir(path).map_err(|error| {
        LanguageError::new(
            ErrorKind::InvalidForm,
            format!("read-dir: failed to read directory {path}: {error}"),
            span,
        )
    })?;
    let mut entries = Vec::new();
    for entry in reader {
        match entry {
            Ok(entry) => entries.push(entry.file_name().to_string_lossy().into_owned()),
            Err(error) => {
                return Err(LanguageError::new(
                    ErrorKind::InvalidForm,
                    format!("read-dir: failed to read an entry in {path}: {error}"),
                    span,
                ))
            }
        }
    }
    Ok(entries)
}

#[cfg(target_arch = "wasm32")]
fn read_dir(_path: &str, span: Span) -> Result<Vec<String>, LanguageError> {
    Err(LanguageError::new(
        ErrorKind::InvalidForm,
        "read-dir: file system access is not available in this build",
        span,
    ))
}

#[cfg(not(target_arch = "wasm32"))]
fn write_file(path: &str, content: &str, span: Span) -> Result<(), LanguageError> {
    std::fs::write(path, content).map_err(|error| {
        LanguageError::new(
            ErrorKind::InvalidForm,
            format!("write-file: failed to write file {path}: {error}"),
            span,
        )
    })
}

#[cfg(target_arch = "wasm32")]
fn write_file(_path: &str, _content: &str, span: Span) -> Result<(), LanguageError> {
    Err(LanguageError::new(
        ErrorKind::InvalidForm,
        "write-file: file system access is not available in this build",
        span,
    ))
}

#[cfg(not(target_arch = "wasm32"))]
fn read_file_bytes(path: &str, span: Span) -> Result<Vec<u8>, LanguageError> {
    std::fs::read(path).map_err(|error| {
        LanguageError::new(
            ErrorKind::InvalidForm,
            format!("read-file-bytes: failed to read file {path}: {error}"),
            span,
        )
    })
}

#[cfg(target_arch = "wasm32")]
fn read_file_bytes(_path: &str, span: Span) -> Result<Vec<u8>, LanguageError> {
    Err(LanguageError::new(
        ErrorKind::InvalidForm,
        "read-file-bytes: file system access is not available in this build",
        span,
    ))
}

#[cfg(not(target_arch = "wasm32"))]
fn write_file_bytes(path: &str, bytes: &[u8], span: Span) -> Result<(), LanguageError> {
    std::fs::write(path, bytes).map_err(|error| {
        LanguageError::new(
            ErrorKind::InvalidForm,
            format!("write-file-bytes: failed to write file {path}: {error}"),
            span,
        )
    })
}

#[cfg(target_arch = "wasm32")]
fn write_file_bytes(_path: &str, _bytes: &[u8], span: Span) -> Result<(), LanguageError> {
    Err(LanguageError::new(
        ErrorKind::InvalidForm,
        "write-file-bytes: file system access is not available in this build",
        span,
    ))
}
