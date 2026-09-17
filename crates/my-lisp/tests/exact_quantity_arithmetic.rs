use my_lisp::{eval_program, Session};

fn eval_exact_quantity(source: &str) -> String {
    let mut session = Session::default();
    for library in [
        include_str!("../../../lib/core.lisp"),
        include_str!("../../../lib/quantity.lisp"),
        include_str!("../../../lib/si.lisp"),
    ] {
        eval_program(library, &mut session).expect("exact quantity library should load");
    }
    eval_program(source, &mut session)
        .unwrap_or_else(|e| panic!("exact quantity expression failed: {e}\nsource: {source}"))
        .value
        .to_string()
}

#[test]
fn speed_of_light_times_one_second_is_exact_distance_and_divides_back() {
    let source = r#"
        (def one-second
          (make-quantity
            1
            (make-unit
              (list (make-dimension (quote second) 1)))))
        (def distance
          (quantity-product
            (scientific-constant-quantity si:defining-speed-of-light)
            one-second))
        (def recovered
          (quantity-quotient distance one-second))
        (list
          distance
          recovered
          (equal?
            recovered
            (scientific-constant-quantity si:defining-speed-of-light)))
    "#;

    // Expected shape queried live from Lisp, not hardcoded (#114/#220).
    let equal_same_shape = eval_exact_quantity("(equal? 1 1)");
    assert_eq!(
        eval_exact_quantity(source),
        format!(
            "((quantity/1 299792458 (unit/1 (dimension/1 metre 1))) (quantity/1 299792458 (unit/1 (dimension/1 metre 1) (dimension/1 second -1))) {equal_same_shape})"
        )
    );
}

#[test]
fn planck_times_cesium_frequency_stays_exact_and_becomes_energy_dimension() {
    let source = r#"
        (def energy
          (quantity-product
            (scientific-constant-quantity si:defining-planck-constant)
            (scientific-constant-quantity si:defining-cesium-frequency)))
        (list
          (quantity-value energy)
          (quantity-unit energy)
          (scientific-constant? energy))
    "#;

    assert_eq!(
        eval_exact_quantity(source),
        "(121822045942277331/20000000000000000000000000000000000000000 (unit/1 (dimension/1 kilogram 1) (dimension/1 metre 2) (dimension/1 second -2)) ())"
    );
}
