//! Exercises the TCP primitives (PLAN.md item 21): tcp-connect/tcp-listen/
//! tcp-accept/tcp-read/tcp-write/tcp-close. The outbound-client half of
//! "talk to other AI systems" (principle 3 extended to LLM APIs/other
//! agents) and the inbound-server half (accepting connections from other
//! agents). Each test runs a server on its own OS thread — a separate
//! `Session`/`Environment` per thread, no `Rc` crosses a thread boundary,
//! same as any two independent my-lisp processes talking over a real
//! socket would be.
//! Перевіряє TCP-примітиви (PLAN.md, пункт 21): tcp-connect/tcp-listen/
//! tcp-accept/tcp-read/tcp-write/tcp-close. Вихідна/клієнтська половина
//! "спілкуватись з іншими AI-системами" (принцип 3, поширений на LLM
//! API/інших агентів) і вхідна/серверна половина (прийом з'єднань від
//! інших агентів). Кожен тест запускає сервер на власному OS-потоці —
//! окремі `Session`/`Environment` на потік, жоден `Rc` не перетинає межу
//! потоку, так само як два незалежні процеси my-lisp, що спілкуються через
//! реальний сокет.

use my_lisp::{eval_program, ErrorKind, Session, Value};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;

/// Grabs a free port by binding to port 0 and reading back what the OS
/// assigned, then immediately releasing it — avoids hardcoding a port
/// number that could collide with another test or a real service.
/// Займає вільний порт, прибіндившись до порту 0 і зчитавши призначений
/// ОС номер, тоді одразу звільняє його — уникає жорстко закодованого
/// номера порту, який міг би зіткнутися з іншим тестом чи реальним сервісом.
/// Runs a client-side my-lisp program, retrying the whole thing a few
/// times if it fails — a guard against exactly one kind of flakiness,
/// not a general retry-until-it-works: the server thread's `tcp-listen`
/// needs a moment to actually bind and start accepting after
/// `thread::spawn` returns, and under a fully parallel `cargo test` run
/// (296 tests, real thread contention) a fixed short sleep isn't always
/// enough. Each retry is a fresh `tcp-connect` attempt; the server's
/// single `tcp-accept` call just waits longer, unaffected either way.
/// Запускає клієнтську my-lisp-програму, повторюючи все кілька разів у
/// разі провалу — захист саме від одного виду нестабільності, не
/// загальний "повторюй, поки не спрацює": `tcp-listen` серверного потоку
/// потребує миті, щоб реально забіндитись і почати приймати з'єднання
/// після повернення з `thread::spawn`, і під повністю паралельним
/// прогоном `cargo test` (296 тестів, реальна конкуренція за потоки)
/// фіксований короткий сон не завжди достатній. Кожна повторна спроба —
/// свіжий виклик `tcp-connect`; єдиний виклик `tcp-accept` сервера просто
/// чекає довше, байдуже в обох випадках.
fn eval_client_with_retry(
    source: &str,
    session: &mut Session,
) -> Result<my_lisp::EvalResult, my_lisp::LanguageError> {
    let mut last_error = None;
    for attempt in 0..20 {
        if attempt > 0 {
            thread::sleep(std::time::Duration::from_millis(100));
        }
        match eval_program(source, session) {
            Ok(result) => return Ok(result),
            // Only a `tcp-connect` failure is safe to retry: the server's
            // single `tcp-accept` hasn't consumed anything yet in that
            // case. Any other error (e.g. something failed *after* a
            // successful connect) must not retry — a second connection
            // attempt would race a server that already accepted-and-
            // exited on the first one, trading a clear failure for a hang.
            // Лише провал `tcp-connect` безпечно повторювати: єдиний
            // `tcp-accept` сервера в цьому випадку ще нічого не спожив.
            // Будь-яка інша помилка (напр. щось провалилось *після*
            // успішного підключення) не має повторюватись — друга спроба
            // з'єднання змагалася б із сервером, що вже прийняв і завершився
            // на першому, міняючи чітку помилку на зависання.
            Err(error) if error.message.contains("tcp-connect:") => last_error = Some(error),
            Err(error) => return Err(error),
        }
    }
    Err(last_error.expect("at least one attempt should have run"))
}

fn free_port() -> u16 {
    TcpListener::bind("127.0.0.1:0")
        .expect("binding to port 0 should succeed")
        .local_addr()
        .expect("a bound listener should have a local address")
        .port()
}

fn load_knowledge(session: &mut Session) {
    eval_program(include_str!("../../../lib/core.my"), session).unwrap();
    eval_program(include_str!("../../../lib/unify.my"), session).unwrap();
    eval_program(include_str!("../../../lib/reason.my"), session).unwrap();
    eval_program(include_str!("../../../lib/forward.my"), session).unwrap();
    eval_program(include_str!("../../../lib/knowledge.my"), session).unwrap();
}

#[test]
fn client_and_server_exchange_one_message_each_way() {
    let port = free_port();

    let server = thread::spawn(move || {
        let mut session = Session::default();
        let source = format!(
            r#"
            (def listener (tcp-listen {port}))
            (def conn (tcp-accept listener))
            (def request (tcp-read conn))
            (tcp-write conn (string-append "echo: " request))
            (tcp-close conn)
            request
            "#
        );
        // `Value` wraps `Rc`, which isn't `Send` — a thread's return value
        // must be, so this converts to an owned `String` before crossing
        // the thread boundary, the same way any two real my-lisp processes
        // would only ever exchange bytes over the socket, never a shared
        // in-memory `Value`.
        // `Value` огортає `Rc`, який не `Send` — значення, що повертає
        // потік, мусить бути, тож тут конвертація в `String` перед межею
        // потоку, так само як два реальні процеси my-lisp обмінювались би
        // лише байтами через сокет, ніколи спільним `Value` у пам'яті.
        eval_program(&source, &mut session)
            .expect("server-side program should evaluate without error")
            .value
            .to_string()
    });

    // Give the server a moment to bind and start listening before the
    // client tries to connect — tcp-connect fails named (not silently)
    // if it loses this race, which would make the test's own failure
    // message point straight at the real cause instead of a hang.
    // Дає серверу момент прибіндитись і почати слухати, перш ніж клієнт
    // спробує підключитись — tcp-connect провалюється названо (не
    // мовчки), якщо програє цю гонку, тож власне повідомлення про
    // провал тесту вкаже прямо на реальну причину, не на зависання.
    thread::sleep(std::time::Duration::from_millis(200));

    let mut client_session = Session::default();
    let client_source = format!(
        r#"
        (def conn (tcp-connect "127.0.0.1" {port}))
        (tcp-write conn "hello from client")
        (def reply (tcp-read conn))
        (tcp-close conn)
        reply
        "#
    );
    let client_result = eval_client_with_retry(&client_source, &mut client_session)
        .expect("client-side program should evaluate without error");

    assert_eq!(
        client_result.value,
        Value::String("echo: hello from client".into())
    );

    // `Value::to_string()` is the `write`/`prin1` form (quoted, escaped —
    // see value.rs's `Display`), not the raw text, so a `Value::String`
    // round-trips as `"hello from client"` with literal quote characters.
    // `Value::to_string()` — це форма `write`/`prin1` (у лапках,
    // екранована — див. `Display` у value.rs), не сирий текст, тож
    // `Value::String` повертається як `"hello from client"` із буквальними
    // символами лапок.
    let server_saw = server.join().expect("server thread should not panic");
    assert_eq!(server_saw, "\"hello from client\"");
}

#[test]
fn send_knowledge_package_transmits_one_canonical_expression_then_eof() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let server = thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let mut text = String::new();
        stream.read_to_string(&mut text).unwrap();
        text
    });
    let mut session = Session::default();
    load_knowledge(&mut session);
    let source = format!(r#"
        (def connection (tcp-connect "127.0.0.1" {port}))
        (send-knowledge-package connection 'exchange
          '(((planet earth)) ((has-mass (var x)) (planet (var x)))))
    "#);
    eval_program(&source, &mut session).unwrap();
    assert_eq!(
        server.join().unwrap(),
        "((format . my-lisp-knowledge) (version 0 1) (module . exchange) (clauses ((planet earth)) ((has-mass (var x)) (planet (var x)))))"
    );
}

#[test]
fn receive_knowledge_package_drains_chunks_and_atomically_imports() {
    let port = free_port();
    let server = thread::spawn(move || {
        let mut session = Session::default();
        load_knowledge(&mut session);
        let source = format!(r#"
            (def listener (tcp-listen {port}))
            (def connection (tcp-accept listener))
            (receive-knowledge-package connection)
            (car (car (reason-in 'exchange '(has-mass earth))))
        "#);
        eval_program(&source, &mut session).unwrap().value.to_string()
    });
    let payload = b"((format . my-lisp-knowledge) (version 0 1) (module . exchange) (clauses . (((planet earth)) ((has-mass (var x)) (planet (var x))))))";
    let mut stream = loop {
        match TcpStream::connect(("127.0.0.1", port)) {
            Ok(stream) => break stream,
            Err(_) => thread::sleep(std::time::Duration::from_millis(50)),
        }
    };
    for chunk in payload.chunks(17) {
        stream.write_all(chunk).unwrap();
    }
    drop(stream);
    assert_eq!(server.join().unwrap(), "(((x . 0) . earth))");
}

#[test]
fn tcp_read_returns_an_empty_string_on_a_closed_connection() {
    let port = free_port();

    let server = thread::spawn(move || {
        let mut session = Session::default();
        let source = format!(
            r#"
            (def listener (tcp-listen {port}))
            (def conn (tcp-accept listener))
            (tcp-close conn)
            "#
        );
        eval_program(&source, &mut session).expect("server-side program should evaluate without error");
    });

    thread::sleep(std::time::Duration::from_millis(200));

    let mut client_session = Session::default();
    let client_source = format!(
        r#"
        (def conn (tcp-connect "127.0.0.1" {port}))
        (tcp-read conn)
        "#
    );
    let result = eval_client_with_retry(&client_source, &mut client_session)
        .expect("reading a closed connection should return an empty string, not error");
    assert_eq!(result.value, Value::String("".into()));

    server.join().expect("server thread should not panic");
}

#[test]
fn tcp_connect_to_a_closed_port_fails_named_not_silently() {
    // A port grabbed and immediately released by free_port() above is very
    // likely to have nothing listening on it in the brief window before
    // the OS could reassign it — connecting there should fail cleanly.
    // Порт, зайнятий і одразу звільнений `free_port()` вище, з високою
    // ймовірністю не має нічого, що слухає, у короткому вікні до того, як
    // ОС могла б перепризначити його — з'єднання туди має провалитись чисто.
    let port = free_port();
    let mut session = Session::default();
    let source = format!(r#"(tcp-connect "127.0.0.1" {port})"#);
    let error = eval_program(&source, &mut session)
        .expect_err("connecting to a port nothing listens on must fail named, not hang or panic");
    assert_eq!(error.kind, ErrorKind::InvalidForm);
}

#[test]
fn tcp_connect_rejects_a_non_string_host() {
    let error = eval_program("(tcp-connect 42 8099)", &mut Session::default())
        .expect_err("a non-string host must fail named, not panic");
    assert_eq!(error.kind, ErrorKind::Type);
}

#[test]
fn tcp_connect_rejects_an_out_of_range_port() {
    let error = eval_program(r#"(tcp-connect "127.0.0.1" 99999)"#, &mut Session::default())
        .expect_err("a port past 65535 must fail named, not panic");
    assert_eq!(error.kind, ErrorKind::Type);
}

#[test]
fn tcp_read_rejects_a_non_connection_argument() {
    let error = eval_program(r#"(tcp-read "not a connection")"#, &mut Session::default())
        .expect_err("a non-connection argument must fail named, not panic");
    assert_eq!(error.kind, ErrorKind::Type);
}

#[test]
fn tcp_write_returns_its_content_argument_unchanged() {
    let port = free_port();
    let server = thread::spawn(move || {
        let mut session = Session::default();
        let source = format!(
            r#"
            (def listener (tcp-listen {port}))
            (def conn (tcp-accept listener))
            (tcp-read conn)
            (tcp-close conn)
            "#
        );
        eval_program(&source, &mut session).expect("server-side program should evaluate without error");
    });

    thread::sleep(std::time::Duration::from_millis(200));

    let mut client_session = Session::default();
    let client_source = format!(
        r#"
        (def conn (tcp-connect "127.0.0.1" {port}))
        (def written (tcp-write conn "payload"))
        (tcp-close conn)
        written
        "#
    );
    let result = eval_client_with_retry(&client_source, &mut client_session)
        .expect("client-side program should evaluate without error");
    assert_eq!(result.value, Value::String("payload".into()));

    server.join().expect("server thread should not panic");
}
