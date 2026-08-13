//! `tcp-connect`/`tcp-listen`/`tcp-accept`/`tcp-read`/`tcp-write`/`tcp-close`
//! (PLAN.md item 21) — "talk to other AI systems" (principle 3, extended to
//! LLM APIs/other agents), the raw byte pipe only: no HTTP/TLS logic lives
//! here, a caller builds that itself with `string-append`/`tcp-write`.

use super::core::exact_arity;
use crate::eval::evaluate;
use crate::{Environment, ErrorKind, Expr, LanguageError, Span, Value};
use std::rc::Rc;

/// `(tcp-connect host port)` — the outbound-client half: opens a TCP
/// connection, returns a `Value::TcpConnection` handle. The caller writes
/// an HTTP request itself with `tcp-write`/`string-append` and reads the
/// response with `tcp-read`; connection failures fail named,
/// `ErrorKind::InvalidForm`, never silently (S2).
pub(crate) fn evaluate_tcp_connect(
    arguments: &[Expr],
    environment: &Environment,
    span: Span,
) -> Result<Value, LanguageError> {
    exact_arity("tcp-connect", arguments, 2, span)?;
    let host_value = evaluate(&arguments[0], environment)?;
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

/// `(tcp-listen port)` — the inbound-server half: binds and starts listening,
/// returns a `Value::TcpListener` handle for `tcp-accept`.
pub(crate) fn evaluate_tcp_listen(
    arguments: &[Expr],
    environment: &Environment,
    span: Span,
) -> Result<Value, LanguageError> {
    exact_arity("tcp-listen", arguments, 1, span)?;
    let port = expect_port(&arguments[0], environment)?;
    let listener = tcp_listen(port, span)?;
    Ok(Value::TcpListener(Rc::new(listener)))
}

/// `(tcp-accept listener)` — blocks until one inbound connection arrives on
/// `listener`, returns it as a `Value::TcpConnection` (the same handle type
/// `tcp-connect` produces).
pub(crate) fn evaluate_tcp_accept(
    arguments: &[Expr],
    environment: &Environment,
    span: Span,
) -> Result<Value, LanguageError> {
    exact_arity("tcp-accept", arguments, 1, span)?;
    let listener_value = evaluate(&arguments[0], environment)?;
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

/// `(tcp-read connection)` — one `read()` call, up to 64 KiB, returned as a
/// string; `""` means the peer closed the connection (EOF), not an error.
pub(crate) fn evaluate_tcp_read(
    arguments: &[Expr],
    environment: &Environment,
    span: Span,
) -> Result<Value, LanguageError> {
    exact_arity("tcp-read", arguments, 1, span)?;
    let connection_value = evaluate(&arguments[0], environment)?;
    let Value::TcpConnection(ref connection) = connection_value else {
        return Err(LanguageError::new(
            ErrorKind::Type,
            "tcp-read expects a TCP connection · tcp-read ochikuie TCP-ziednannia · tcp-read erwartet eine TCP-Verbindung",
            arguments[0].span,
        ));
    };
    let text = tcp_read(connection, span)?;
    Ok(Value::String(Rc::from(text.as_str())))
}

/// `(tcp-write connection content)` — writes `content`'s UTF-8 bytes,
/// returns `content` unchanged (composes like `print`/`write-file`).
pub(crate) fn evaluate_tcp_write(
    arguments: &[Expr],
    environment: &Environment,
    span: Span,
) -> Result<Value, LanguageError> {
    exact_arity("tcp-write", arguments, 2, span)?;
    let connection_value = evaluate(&arguments[0], environment)?;
    let Value::TcpConnection(ref connection) = connection_value else {
        return Err(LanguageError::new(
            ErrorKind::Type,
            "tcp-write expects a TCP connection · tcp-write ochikuie TCP-ziednannia · tcp-write erwartet eine TCP-Verbindung",
            arguments[0].span,
        ));
    };
    let content_value = evaluate(&arguments[1], environment)?;
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

/// `(tcp-close connection)` — explicitly shuts down both directions of the
/// connection rather than waiting for the handle to be dropped, so the
/// peer sees the close promptly. Returns `t`.
pub(crate) fn evaluate_tcp_close(
    arguments: &[Expr],
    environment: &Environment,
    span: Span,
) -> Result<Value, LanguageError> {
    exact_arity("tcp-close", arguments, 1, span)?;
    let connection_value = evaluate(&arguments[0], environment)?;
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
    let value = evaluate(expr, environment)?;
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
    listener
        .accept()
        .map(|(stream, _addr)| stream)
        .map_err(|error| {
            LanguageError::new(
                ErrorKind::InvalidForm,
                format!("tcp-accept: failed to accept a connection: {error}"),
                span,
            )
        })
}

fn tcp_read(
    connection: &std::cell::RefCell<std::net::TcpStream>,
    span: Span,
) -> Result<String, LanguageError> {
    use std::io::Read;
    let mut buffer = [0u8; 65536];
    let read = connection
        .borrow_mut()
        .read(&mut buffer)
        .map_err(|error| {
            LanguageError::new(
                ErrorKind::InvalidForm,
                format!("tcp-read: failed to read from the connection: {error}"),
                span,
            )
        })?;
    String::from_utf8(buffer[..read].to_vec()).map_err(|error| {
        LanguageError::new(
            ErrorKind::InvalidForm,
            format!("tcp-read: received bytes that aren't valid UTF-8: {error}"),
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
