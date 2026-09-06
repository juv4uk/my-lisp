use my_lisp::{eval_program, load_core_library, Session};

#[test]
fn utc_now_returns_utc_calendar_with_nanosecond_field() {
    let mut session = Session::default();
    load_core_library(&mut session).unwrap();
    let value = eval_program("(utc-now)", &mut session)
        .unwrap()
        .value
        .to_string();
    let fields: Vec<&str> = value.trim_matches(['(', ')']).split_whitespace().collect();
    assert_eq!(fields.len(), 8, "utc-now shape: {value}");
    assert_eq!(fields[0], "utc");
    let month: i64 = fields[2].parse().unwrap();
    let day: i64 = fields[3].parse().unwrap();
    let hour: i64 = fields[4].parse().unwrap();
    let minute: i64 = fields[5].parse().unwrap();
    let second: i64 = fields[6].parse().unwrap();
    let nanosecond: i64 = fields[7].parse().unwrap();
    assert!((1..=12).contains(&month));
    assert!((1..=31).contains(&day));
    assert!((0..24).contains(&hour));
    assert!((0..60).contains(&minute));
    assert!((0..60).contains(&second));
    assert!((0..1_000_000_000).contains(&nanosecond));
}

#[test]
fn timezone_detection_is_explicit_and_ntp_requires_host_string() {
    let mut session = Session::default();
    load_core_library(&mut session).unwrap();
    eval_program(include_str!("../../../lib/time.my"), &mut session).unwrap();
    let timezone = eval_program("(timezone-detect)", &mut session)
        .unwrap()
        .value
        .to_string();
    assert!(timezone.starts_with("(detected ") || timezone.starts_with("(unknown "));
    assert!(eval_program("(internet-time-sync 123 100)", &mut session).is_err());
    assert_eq!(
        eval_program("(timezone-config \"Europe/Kyiv\" 7200)", &mut session)
            .unwrap()
            .value
            .to_string(),
        "(accepted (timezone \"Europe/Kyiv\" 7200))"
    );
}
