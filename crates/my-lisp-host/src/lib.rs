//! my-lisp-host - the OS capability layer for my-lisp.
//!
//! This crate owns every piece of code that touches the outside machine:
//! filesystem (`read-file`, `write-file`, `read-file-bytes`,
//! `write-file-bytes`, `read-dir`), raw subprocess execution
//! (`process-run-raw` with its strict allowlist), and TCP sockets. It installs
//! them into the canonical core's capability registry at startup via [`install`].
//!
//! The core crate itself contains none of this: a build that never calls
//! [`install`] has no OS capabilities at all, and evaluating e.g.
//! `(read-file "x")` then fails `UnknownSymbol` like any unbound name.

use my_lisp::{
    eval_expr, exact_arity, register_capability, Environment, ErrorKind, Exactness, Expr,
    LanguageError, Span, Value,
};
use std::rc::Rc;

mod process_raw;

// File-system primitives: `read-file`/`write-file` and their byte-level
// counterparts (`read-file-bytes`/`write-file-bytes`), plus the raw
// `read_file`/`write_file`/`read_file_bytes`/`write_file_bytes` host calls
// `evaluate_load` (in `io`) also needs — kept `pub(super)` so both
// submodules can share the one path to the filesystem rather than each
// having its own.

/// The write-side counterpart to `read-file` (PLAN.md item 13) — one
/// primitive that opens and writes in a single step, the same shape
/// `read-file` already uses for opening and reading, rather than a
/// separate stateful file-handle value: the language has no mutable
/// cells or handles to represent one, and none of `read-file`/`load`
/// needed one either. Always creates or truncates-and-overwrites the
/// target file (`std::fs::write`'s own semantics), never appends —
/// append is a separate, not-yet-decided capability, not silently
/// folded into this one.
fn evaluate_read_file(
    arguments: &[Expr],
    environment: &Environment,
    span: Span,
) -> Result<Value, LanguageError> {
    exact_arity("read-file", arguments, 1, span)?;
    let evaluated = eval_expr(&arguments[0], environment)?;
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
fn evaluate_read_dir(
    arguments: &[Expr],
    environment: &Environment,
    span: Span,
) -> Result<Value, LanguageError> {
    exact_arity("read-dir", arguments, 1, span)?;
    let evaluated = eval_expr(&arguments[0], environment)?;
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

fn evaluate_write_file(
    arguments: &[Expr],
    environment: &Environment,
    span: Span,
) -> Result<Value, LanguageError> {
    exact_arity("write-file", arguments, 2, span)?;
    let path_value = eval_expr(&arguments[0], environment)?;
    let Value::String(ref path) = path_value else {
        return Err(LanguageError::new(
            ErrorKind::Type,
            "write-file expects a string path · write-file ochikuie riadok-shliakh · write-file erwartet einen String-Pfad",
            span,
        ));
    };
    let content_value = eval_expr(&arguments[1], environment)?;
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
fn evaluate_write_file_bytes(
    arguments: &[Expr],
    environment: &Environment,
    span: Span,
) -> Result<Value, LanguageError> {
    exact_arity("write-file-bytes", arguments, 2, span)?;
    let path_value = eval_expr(&arguments[0], environment)?;
    let Value::String(ref path) = path_value else {
        return Err(LanguageError::new(
            ErrorKind::Type,
            "write-file-bytes expects a string path · write-file-bytes ochikuie riadok-shliakh · write-file-bytes erwartet einen String-Pfad",
            span,
        ));
    };
    let bytes_value = eval_expr(&arguments[1], environment)?;
    let bytes = expect_byte_list(&bytes_value, arguments[1].span)?;
    write_file_bytes(path, &bytes, span)?;
    Ok(bytes_value)
}

/// `(read-file-bytes path)` (PLAN.md item 22) — the byte-level counterpart
/// to `read-file`: returns the file's raw bytes as a list of fixnums
/// 0-255, not a UTF-8-decoded string, which would fail outright — or
/// worse, silently corrupt — on a non-UTF-8 file.
fn evaluate_read_file_bytes(
    arguments: &[Expr],
    environment: &Environment,
    span: Span,
) -> Result<Value, LanguageError> {
    exact_arity("read-file-bytes", arguments, 1, span)?;
    let evaluated = eval_expr(&arguments[0], environment)?;
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

fn expect_tcp_byte_list(value: &Value, span: Span) -> Result<Vec<u8>, LanguageError> {
    let mut bytes = Vec::new();
    let mut current = value;
    loop {
        match current {
            Value::Nil => return Ok(bytes),
            Value::Pair(head, tail) => {
                let Value::Number(number, _) = **head else {
                    return Err(LanguageError::new(
                        ErrorKind::Type,
                        "tcp-write-raw expects a list of integers 0-255 · tcp-write-raw ochikuie spysok tsilykh chysel 0-255 · tcp-write-raw erwartet eine Liste von Ganzzahlen 0-255",
                        span,
                    ));
                };
                if number.fract() != 0.0 || !(0.0..=255.0).contains(&number) {
                    return Err(LanguageError::new(
                        ErrorKind::Type,
                        "tcp-write-raw expects each element to be an integer between 0 and 255 · tcp-write-raw ochikuie, shchob kozhen element buv tsilym chyslom vid 0 do 255 · tcp-write-raw erwartet, dass jedes Element eine Ganzzahl zwischen 0 und 255 ist",
                        span,
                    ));
                }
                bytes.push(number as u8);
                current = tail;
            }
            _ => {
                return Err(LanguageError::new(
                    ErrorKind::Type,
                    "tcp-write-raw expects a proper list of integers 0-255 · tcp-write-raw ochikuie pravylnyi spysok tsilykh chysel 0-255 · tcp-write-raw erwartet eine echte Liste von Ganzzahlen 0-255",
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

// TCP handles and byte transport only. Text encoding/decoding is language-owned.

fn evaluate_tcp_connect(
    arguments: &[Expr],
    environment: &Environment,
    span: Span,
) -> Result<Value, LanguageError> {
    exact_arity("tcp-connect", arguments, 2, span)?;
    let host_value = eval_expr(&arguments[0], environment)?;
    let Value::String(ref host) = host_value else {
        return Err(LanguageError::new(
            ErrorKind::Type,
            "tcp-connect expects a string host · tcp-connect ochikuie riadok-khost · tcp-connect erwartet einen String-Host",
            arguments[0].span,
        ));
    };
    let port = expect_port(&arguments[1], environment)?;
    let stream = tcp_connect(host, port, span)?;
    Ok(Value::TcpConnection(Rc::new(std::cell::RefCell::new(stream))))
}

fn evaluate_tcp_listen(
    arguments: &[Expr],
    environment: &Environment,
    span: Span,
) -> Result<Value, LanguageError> {
    exact_arity("tcp-listen", arguments, 1, span)?;
    let port = expect_port(&arguments[0], environment)?;
    let listener = tcp_listen(port, span)?;
    Ok(Value::TcpListener(Rc::new(listener)))
}

fn evaluate_tcp_accept(
    arguments: &[Expr],
    environment: &Environment,
    span: Span,
) -> Result<Value, LanguageError> {
    exact_arity("tcp-accept", arguments, 1, span)?;
    let listener_value = eval_expr(&arguments[0], environment)?;
    let Value::TcpListener(ref listener) = listener_value else {
        return Err(LanguageError::new(
            ErrorKind::Type,
            "tcp-accept expects a TCP listener · tcp-accept ochikuie TCP-listener · tcp-accept erwartet einen TCP-Listener",
            arguments[0].span,
        ));
    };
    let stream = tcp_accept(listener, span)?;
    Ok(Value::TcpConnection(Rc::new(std::cell::RefCell::new(stream))))
}

/// `(tcp-read-raw connection)` performs one read of up to 64 KiB and returns
/// the exact bytes as a proper list of exact integers 0..255. Empty list is
/// EOF. No UTF-8 interpretation occurs in the host.
fn evaluate_tcp_read_raw(
    arguments: &[Expr],
    environment: &Environment,
    span: Span,
) -> Result<Value, LanguageError> {
    exact_arity("tcp-read-raw", arguments, 1, span)?;
    let connection_value = eval_expr(&arguments[0], environment)?;
    let Value::TcpConnection(ref connection) = connection_value else {
        return Err(LanguageError::new(
            ErrorKind::Type,
            "tcp-read-raw expects a TCP connection · tcp-read-raw ochikuie TCP-ziednannia · tcp-read-raw erwartet eine TCP-Verbindung",
            arguments[0].span,
        ));
    };
    let bytes = tcp_read_raw(connection, span)?;
    Ok(Value::list(
        bytes
            .into_iter()
            .map(|byte| Value::Number(byte as f64, Exactness::Exact)),
    ))
}

/// Transitional raw write boundary. The host accepts already-interpreted bytes
/// and performs only the socket write. The legacy text `tcp-write` remains
/// registered until the language-owned wrapper is exercised by CI.
fn evaluate_tcp_write_raw(
    arguments: &[Expr],
    environment: &Environment,
    span: Span,
) -> Result<Value, LanguageError> {
    exact_arity("tcp-write-raw", arguments, 2, span)?;
    let connection_value = eval_expr(&arguments[0], environment)?;
    let Value::TcpConnection(ref connection) = connection_value else {
        return Err(LanguageError::new(
            ErrorKind::Type,
            "tcp-write-raw expects a TCP connection · tcp-write-raw ochikuie TCP-ziednannia · tcp-write-raw erwartet eine TCP-Verbindung",
            arguments[0].span,
        ));
    };
    let bytes_value = eval_expr(&arguments[1], environment)?;
    let bytes = expect_tcp_byte_list(&bytes_value, arguments[1].span)?;
    tcp_write_raw(connection, &bytes, span)?;
    Ok(bytes_value)
}

fn evaluate_tcp_write(
    arguments: &[Expr],
    environment: &Environment,
    span: Span,
) -> Result<Value, LanguageError> {
    exact_arity("tcp-write", arguments, 2, span)?;
    let connection_value = eval_expr(&arguments[0], environment)?;
    let Value::TcpConnection(ref connection) = connection_value else {
        return Err(LanguageError::new(
            ErrorKind::Type,
            "tcp-write expects a TCP connection · tcp-write ochikuie TCP-ziednannia · tcp-write erwartet eine TCP-Verbindung",
            arguments[0].span,
        ));
    };
    let content_value = eval_expr(&arguments[1], environment)?;
    let Value::String(ref content) = content_value else {
        return Err(LanguageError::new(
            ErrorKind::Type,
            "tcp-write expects a string as its second argument · tcp-write ochikuie riadok druhym arhumentom · tcp-write erwartet eine Zeichenkette als zweites Argument",
            arguments[1].span,
        ));
    };
    tcp_write(connection, content, span)?;
    Ok(content_value)
}

fn evaluate_tcp_close(
    arguments: &[Expr],
    environment: &Environment,
    span: Span,
) -> Result<Value, LanguageError> {
    exact_arity("tcp-close", arguments, 1, span)?;
    let connection_value = eval_expr(&arguments[0], environment)?;
    let Value::TcpConnection(ref connection) = connection_value else {
        return Err(LanguageError::new(
            ErrorKind::Type,
            "tcp-close expects a TCP connection · tcp-close ochikuie TCP-ziednannia · tcp-close erwartet eine TCP-Verbindung",
            arguments[0].span,
        ));
    };
    tcp_close(connection, span)?;
    Ok(Value::Bool(true))
}

fn expect_port(expr: &Expr, environment: &Environment) -> Result<u16, LanguageError> {
    let value = eval_expr(expr, environment)?;
    let Value::Number(port, _) = value else {
        return Err(LanguageError::new(
            ErrorKind::Type,
            "expected a port number · ochikuvavsia nomer portu · erwartete eine Portnummer",
            expr.span,
        ));
    };
    if port.fract() != 0.0 || port < 0.0 || port > u16::MAX as f64 {
        return Err(LanguageError::new(
            ErrorKind::Type,
            "port must be an integer between 0 and 65535 · port maie buty tsilym chyslom vid 0 do 65535 · Port muss eine Ganzzahl zwischen 0 und 65535 sein",
            expr.span,
        ));
    }
    Ok(port as u16)
}

#[cfg(not(target_arch = "wasm32"))]
fn tcp_connect(host: &str, port: u16, span: Span) -> Result<std::net::TcpStream, LanguageError> {
    std::net::TcpStream::connect((host, port)).map_err(|error| {
        LanguageError::new(
            ErrorKind::InvalidForm,
            format!("tcp-connect: failed to connect to {host}:{port}: {error}"),
            span,
        )
    })
}

#[cfg(target_arch = "wasm32")]
fn tcp_connect(_host: &str, _port: u16, span: Span) -> Result<std::net::TcpStream, LanguageError> {
    Err(LanguageError::new(
        ErrorKind::InvalidForm,
        "tcp-connect: networking is not available in this build",
        span,
    ))
}

#[cfg(not(target_arch = "wasm32"))]
fn tcp_listen(port: u16, span: Span) -> Result<std::net::TcpListener, LanguageError> {
    std::net::TcpListener::bind(("0.0.0.0", port)).map_err(|error| {
        LanguageError::new(
            ErrorKind::InvalidForm,
            format!("tcp-listen: failed to bind port {port}: {error}"),
            span,
        )
    })
}

#[cfg(target_arch = "wasm32")]
fn tcp_listen(_port: u16, span: Span) -> Result<std::net::TcpListener, LanguageError> {
    Err(LanguageError::new(
        ErrorKind::InvalidForm,
        "tcp-listen: networking is not available in this build",
        span,
    ))
}

fn tcp_accept(
    listener: &std::net::TcpListener,
    span: Span,
) -> Result<std::net::TcpStream, LanguageError> {
    listener.accept().map(|(stream, _)| stream).map_err(|error| {
        LanguageError::new(
            ErrorKind::InvalidForm,
            format!("tcp-accept: failed to accept a connection: {error}"),
            span,
        )
    })
}

fn tcp_read_raw(
    connection: &std::cell::RefCell<std::net::TcpStream>,
    span: Span,
) -> Result<Vec<u8>, LanguageError> {
    use std::io::Read;
    let mut buffer = [0u8; 65536];
    let read = connection.borrow_mut().read(&mut buffer).map_err(|error| {
        LanguageError::new(
            ErrorKind::InvalidForm,
            format!("tcp-read-raw: failed to read from the connection: {error}"),
            span,
        )
    })?;
    Ok(buffer[..read].to_vec())
}

fn tcp_write_raw(
    connection: &std::cell::RefCell<std::net::TcpStream>,
    bytes: &[u8],
    span: Span,
) -> Result<(), LanguageError> {
    use std::io::Write;
    connection.borrow_mut().write_all(bytes).map_err(|error| {
        LanguageError::new(
            ErrorKind::InvalidForm,
            format!("tcp-write-raw: failed to write to the connection: {error}"),
            span,
        )
    })
}

fn tcp_write(
    connection: &std::cell::RefCell<std::net::TcpStream>,
    content: &str,
    span: Span,
) -> Result<(), LanguageError> {
    use std::io::Write;
    connection
        .borrow_mut()
        .write_all(content.as_bytes())
        .map_err(|error| {
            LanguageError::new(
                ErrorKind::InvalidForm,
                format!("tcp-write: failed to write to the connection: {error}"),
                span,
            )
        })
}

fn tcp_close(
    connection: &std::cell::RefCell<std::net::TcpStream>,
    span: Span,
) -> Result<(), LanguageError> {
    connection
        .borrow()
        .shutdown(std::net::Shutdown::Both)
        .map_err(|error| {
            LanguageError::new(
                ErrorKind::InvalidForm,
                format!("tcp-close: failed to close the connection: {error}"),
                span,
            )
        })
}

fn evaluate_load(
    arguments: &[Expr],
    environment: &Environment,
    span: Span,
) -> Result<Value, LanguageError> {
    exact_arity("load", arguments, 1, span)?;
    let evaluated = my_lisp::eval_expr(&arguments[0], environment)?;
    let Value::String(ref path) = evaluated else {
        return Err(LanguageError::new(
            ErrorKind::Type,
            "load expects a string path / load ochikuie riadok-shliakh / load erwartet einen String-Pfad",
            span,
        ));
    };

    let source = std::fs::read_to_string(path.as_ref()).map_err(|error| {
        LanguageError::new(
            ErrorKind::InvalidForm,
            format!("load: failed to read file {path}: {error}"),
            span,
        )
    })?;
    let expressions = my_lisp::parse(&source).map_err(|mut error| {
        error.span = span;
        error
    })?;

    let mut last_value = Value::Nil;
    for expr in expressions {
        last_value = my_lisp::eval_expr(&expr, environment)?;
    }

    Ok(last_value)
}

pub fn install() {
    register_capability("read-file", evaluate_read_file);
    register_capability("read-dir", evaluate_read_dir);
    register_capability("read-file-bytes", evaluate_read_file_bytes);
    register_capability("write-file", evaluate_write_file);
    register_capability("write-file-bytes", evaluate_write_file_bytes);
    register_capability("process-run-raw", process_raw::evaluate_process_run_raw);
    register_capability("load", evaluate_load);
    register_capability("tcp-connect", evaluate_tcp_connect);
    register_capability("tcp-listen", evaluate_tcp_listen);
    register_capability("tcp-accept", evaluate_tcp_accept);
    register_capability("tcp-read-raw", evaluate_tcp_read_raw);
    register_capability("tcp-write-raw", evaluate_tcp_write_raw);
    register_capability("tcp-write", evaluate_tcp_write);
    register_capability("tcp-close", evaluate_tcp_close);
}

#[cfg(test)]
mod install_tests {
    #[test]
    fn install_registers_every_host_form() {
        super::install();
        let installed = my_lisp::installed_capabilities();
        for name in [
            "read-file",
            "read-dir",
            "read-file-bytes",
            "write-file",
            "write-file-bytes",
            "process-run-raw",
            "tcp-connect",
            "tcp-listen",
            "tcp-accept",
            "tcp-read-raw",
            "tcp-write-raw",
            "tcp-write",
            "tcp-close",
        ] {
            assert!(installed.iter().any(|n| n == name), "{name} not registered");
        }
    }
}
