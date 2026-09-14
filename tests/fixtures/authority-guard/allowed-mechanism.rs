#[test]
fn host_mechanism_assertion_is_allowed() {
    let frame = vec![1_u8, 2, 3, 4];
    assert_eq!(frame.len(), 4);
}
