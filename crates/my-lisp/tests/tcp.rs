//! Exercises the TCP primitives (PLAN.md item 21): tcp-connect/tcp-listen/
//! tcp-accept/tcp-read/tcp-write/tcp-close. The outbound-client half of
//! "talk to other AI systems" (principle 3 extended to LLM APIs/other
//! agents) and the inbound-server half (accepting connections from other
//! agents). Each test runs a server on its own OS thread — a separate
//! `Session`/`Environment` per thread, no `Rc` crosses a thread boundary,
//! same as any two independent my-lisp processes talking over a real
//! socket would be.
//! Pereviriaie TCP-prymityvy (PLAN.md, punkt 21): tcp-connect/tcp-listen/
//! tcp-accept/tcp-read/tcp-write/tcp-close. Vykhidna/kliientska polovyna
//! "spilkuvatys z inshymy AI-systemamy" (pryntsyp 3, poshyrenyi na LLM
//! API/inshykh ahentiv) i vkhidna/serverna polovyna (pryiom ziednan vid
//! inshykh ahentiv). Kozhen test zapuskaie server na vlasnomu OS-pototsi —
//! okremi `Session`/`Environment` na potik, zhoden `Rc` ne peretynaie mezhu
//! potoku, tak samo yak dva nezalezhni protsesy my-lisp, shcho spilkuiutsia cherez
//! realnyi soket.

use my_lisp::{eval_program, ErrorKind, Session, Value};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;

/// Grabs a free port by binding to port 0 and reading back what the OS
/// assigned, then immediately releasing it — avoids hardcoding a port
/// number that could collide with another test or a real service.
/// Zaimaie vilnyi port, prybindyvshys do portu 0 i zchytavshy pryznachenyi
/// OS nomer, todi odrazu zvilniaie yoho — unykaie zhorstko zakodovanoho
/// nomera portu, yakyi mih by zitknutysia z inshym testom chy realnym servisom.
/// Runs a client-side my-lisp program, retrying the whole thing a few
/// times if it fails — a guard against exactly one kind of flakiness,
/// not a general retry-until-it-works: the server thread's `tcp-listen`
/// needs a moment to actually bind and start accepting after
/// `thread::spawn` returns, and under a fully parallel `cargo test` run
/// (296 tests, real thread contention) a fixed short sleep isn't always
/// enough. Each retry is a fresh `tcp-connect` attempt; the server's
/// single `tcp-accept` call just waits longer, unaffected either way.
/// Zapuskaie kliientsku my-lisp-prohramu, povtoriuiuchy vse kilka raziv u
/// razi provalu — zakhyst same vid odnoho vydu nestabilnosti, ne
/// zahalnyi "povtoriui, poky ne spratsiuie": `tcp-listen` servernoho potoku
/// potrebuie myti, shchob realno zabindytys i pochaty pryimaty ziednannia
/// pislia povernennia z `thread::spawn`, i pid povnistiu paralelnym
/// prohonom `cargo test` (296 testiv, realna konkurentsiia za potoky)
/// fiksovanyi korotkyi son ne zavzhdy dostatnii. Kozhna povtorna sproba —
/// svizhyi vyklyk `tcp-connect`; yedynyi vyklyk `tcp-accept` servera prosto
/// chekaie dovshe, baiduzhe v obokh vypadkakh.
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
            // Lyshe proval `tcp-connect` bezpechno povtoriuvaty: yedynyi
            // `tcp-accept` servera v tsomu vypadku shche nichoho ne spozhyv.
            // Bud-yaka insha pomylka (napr. shchos provalylos *pislia*
            // uspishnoho pidkliuchennia) ne maie povtoriuvatys — druha sproba
            // ziednannia zmahalasia b iz serverom, shcho vzhe pryiniav i zavershyvsia
            // na pershomu, miniaiuchy chitku pomylku na zavysannia.
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
        // `Value` ohortaie `Rc`, yakyi ne `Send` — znachennia, shcho povertaie
        // potik, musyt buty, tozh tut konvertatsiia v `String` pered mezheiu
        // potoku, tak samo yak dva realni protsesy my-lisp obminiuvalys by
        // lyshe baitamy cherez soket, nikoly spilnym `Value` u pamiati.
        eval_program(&source, &mut session)
            .expect("server-side program should evaluate without error")
            .value
            .to_string()
    });

    // Give the server a moment to bind and start listening before the
    // client tries to connect — tcp-connect fails named (not silently)
    // if it loses this race, which would make the test's own failure
    // message point straight at the real cause instead of a hang.
    // Daie serveru moment prybindytys i pochaty slukhaty, persh nizh kliient
    // sprobuie pidkliuchytys — tcp-connect provaliuietsia nazvano (ne
    // movchky), yakshcho prohraie tsiu honku, tozh vlasne povidomlennia pro
    // proval testu vkazhe priamo na realnu prychynu, ne na zavysannia.
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
    // `Value::to_string()` — tse forma `write`/`prin1` (u lapkakh,
    // ekranovana — dyv. `Display` u value.rs), ne syryi tekst, tozh
    // `Value::String` povertaietsia yak `"hello from client"` iz bukvalnymy
    // symvolamy lapok.
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
fn framed_exchange_returns_an_accepted_receipt_to_the_sender() {
    let port = free_port();
    let server = thread::spawn(move || {
        let mut session = Session::default();
        load_knowledge(&mut session);
        let source = format!(r#"
            (def listener (tcp-listen {port}))
            (def connection (tcp-accept listener))
            (accept-knowledge-exchange connection)
        "#);
        eval_program(&source, &mut session).unwrap().value.to_string()
    });
    let mut client = Session::default();
    load_knowledge(&mut client);
    let source = format!(r#"
        (def connection (tcp-connect "127.0.0.1" {port}))
        (exchange-knowledge-package connection 'exchange '(((planet earth))))
    "#);
    let receipt = eval_client_with_retry(&source, &mut client).unwrap();
    assert_eq!(receipt.value.to_string(),
               "(accepted (module exchange) (knowledge (((planet earth)))))");
    assert_eq!(server.join().unwrap(), receipt.value.to_string());
}

#[test]
fn framed_exchange_returns_conflict_and_does_not_install_the_new_fact() {
    let port = free_port();
    let server = thread::spawn(move || {
        let mut session = Session::default();
        load_knowledge(&mut session);
        let source = format!(r#"
            (defmodule exchange '(((not (planet pluto)))))
            (def listener (tcp-listen {port}))
            (def connection (tcp-accept listener))
            (def decision (accept-knowledge-exchange connection))
            (list (car decision) (reason-in 'exchange '(planet pluto)))
        "#);
        eval_program(&source, &mut session).unwrap().value.to_string()
    });
    let mut client = Session::default();
    load_knowledge(&mut client);
    let source = format!(r#"
        (def connection (tcp-connect "127.0.0.1" {port}))
        (exchange-knowledge-package connection 'exchange '(((planet pluto))))
    "#);
    let receipt = eval_client_with_retry(&source, &mut client).unwrap();
    assert_eq!(receipt.value.to_string().split_whitespace().next(), Some("(conflict"));
    assert_eq!(server.join().unwrap(), "(conflict ())");
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
    // Port, zainiatyi i odrazu zvilnenyi `free_port()` vyshche, z vysokoiu
    // ymovirnistiu ne maie nichoho, shcho slukhaie, u korotkomu vikni do toho, yak
    // OS mohla b perepryznachyty yoho — ziednannia tudy maie provalytys chysto.
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
