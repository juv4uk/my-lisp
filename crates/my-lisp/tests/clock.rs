use my_lisp::{eval_program, Session};

#[test]
fn utc_now_returns_utc_calendar_with_nanosecond_field() {
    let mut session = Session::default();
    eval_program(include_str!("../../../lib/core.my"), &mut session).unwrap();
    let value = eval_program("(utc-now)", &mut session).unwrap().value.to_string();
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
